use std::collections::BTreeSet;
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

const EXPECTED_STDOUT: &str = "representative-rail-ok";
const BWRAP_SKIP: &str = "SKIP: representative rail offline Cargo smoke requires Linux + bwrap + /nix/store";
const FAILING_CARGO_SHIM: &str = "#!/bin/sh\necho cargo must not be invoked >&2\nexit 99\n";
const EXECUTABLE_MODE: u32 = 0o755;
const BLAKE3_HEX_BYTES: usize = 64;
const MATRIX_ID: &str = "representative-rust-compatibility-v1";
const MATRIX_PATH: &str = "examples/rust_compatibility_surface_matrix.ncl";
const BLOCKER_MISSING_VENDOR: &str = "missing-vendored-registry-source";
const BLOCKER_STALE_LOCK: &str = "stale-lockfile-digest";
const BLOCKER_BUILD_METADATA: &str = "malformed-build-script-metadata";
const BLOCKER_PROC_MACRO: &str = "missing-proc-macro-host-artifact";
const BLOCKER_NATIVE_LINK: &str = "unsupported-native-link-metadata";
const BLOCKER_FEATURE_SURFACE: &str = "missing-feature-surface";
const BLOCKER_TARGET_CFG_SURFACE: &str = "missing-target-cfg-surface";
const BLOCKER_WORKSPACE_INHERITANCE: &str = "missing-workspace-inheritance-surface";
const BLOCKER_MULTI_PACKAGE_BINARY: &str = "missing-multi-package-binary-surface";
const BLOCKER_VENDORED_GIT: &str = "unsupported-vendored-git-source";
const BLOCKER_PKG_CONFIG: &str = "unsupported-pkg-config";
const BLOCKER_LINK_METADATA: &str = "unsupported-rustc-link-metadata";
const BLOCKER_NATIVE_C: &str = "unsupported-native-c-compile";

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn can_build() -> bool {
    Path::new("/nix/store").exists()
        && std::process::Command::new("bwrap")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
}

fn write_representative_workspace(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("local-lib/src")).unwrap();
    std::fs::create_dir_all(root.join("feature-lib/src")).unwrap();
    std::fs::create_dir_all(root.join("target-helper/src")).unwrap();
    std::fs::create_dir_all(root.join("compat-macro/src")).unwrap();
    std::fs::create_dir_all(root.join("compat-tools/src")).unwrap();
    std::fs::create_dir_all(root.join("vendor/serde/src")).unwrap();
    std::fs::create_dir_all(root.join("vendor/git-helper/src")).unwrap();
    std::fs::create_dir_all(root.join(".cargo")).unwrap();
    std::fs::write(root.join("Cargo.toml"), representative_manifest()).unwrap();
    std::fs::write(root.join("Cargo.lock"), representative_lockfile()).unwrap();
    std::fs::write(root.join("build.rs"), representative_build_rs()).unwrap();
    std::fs::write(root.join("src/main.rs"), representative_main_rs()).unwrap();
    std::fs::write(root.join("local-lib/Cargo.toml"), local_lib_manifest()).unwrap();
    std::fs::write(root.join("local-lib/src/lib.rs"), "pub fn message() -> &'static str { \"local\" }\n").unwrap();
    std::fs::write(root.join("feature-lib/Cargo.toml"), feature_lib_manifest()).unwrap();
    std::fs::write(root.join("feature-lib/src/lib.rs"), "pub fn feature_enabled() -> bool { true }\n").unwrap();
    std::fs::write(root.join("target-helper/Cargo.toml"), target_helper_manifest()).unwrap();
    std::fs::write(root.join("target-helper/src/lib.rs"), "pub fn target_marker() -> &'static str { \"unix\" }\n")
        .unwrap();
    std::fs::write(root.join("compat-macro/Cargo.toml"), proc_macro_manifest()).unwrap();
    std::fs::write(root.join("compat-macro/src/lib.rs"), proc_macro_lib_rs()).unwrap();
    std::fs::write(root.join("compat-tools/Cargo.toml"), compat_tools_manifest()).unwrap();
    std::fs::write(root.join("compat-tools/src/main.rs"), "fn main() { println!(\"compat-tool\"); }\n").unwrap();
    std::fs::write(root.join("vendor/serde/Cargo.toml"), vendor_manifest()).unwrap();
    std::fs::write(root.join("vendor/serde/src/lib.rs"), "pub fn marker() {}\n").unwrap();
    std::fs::write(root.join("vendor/git-helper/Cargo.toml"), git_helper_manifest()).unwrap();
    std::fs::write(root.join("vendor/git-helper/src/lib.rs"), "pub fn git_marker() {}\n").unwrap();
    std::fs::write(root.join(".cargo/config.toml"), cargo_config()).unwrap();
}

fn representative_manifest() -> &'static str {
    r#"[workspace]
members = ["local-lib", "feature-lib", "target-helper", "compat-macro", "compat-tools"]
resolver = "2"

[workspace.package]
edition = "2021"

[package]
name = "compat-demo"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[[bin]]
name = "compat-demo"
path = "src/main.rs"

[features]
default = ["feature-lib/extra"]
matrix-feature = ["feature-lib/extra"]

[dependencies]
local-lib = { path = "local-lib" }
feature-lib = { path = "feature-lib", optional = true, default-features = false, features = ["extra"] }
compat-macro = { path = "compat-macro" }
serde = "1.0.0"

[target.'cfg(unix)'.dependencies]
target-helper = { path = "target-helper" }
"#
}

fn representative_lockfile() -> &'static str {
    r#"# This file is automatically @generated by Cargo.
version = 4

[[package]]
name = "compat-demo"
version = "0.1.0"
dependencies = ["compat-macro", "feature-lib", "local-lib", "serde", "target-helper"]

[[package]]
name = "compat-macro"
version = "0.1.0"

[[package]]
name = "compat-tools"
version = "0.1.0"

[[package]]
name = "feature-lib"
version = "0.1.0"

[[package]]
name = "local-lib"
version = "0.1.0"

[[package]]
name = "target-helper"
version = "0.1.0"

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#
}

fn representative_build_rs() -> &'static str {
    "fn main() { println!(\"cargo:rustc-env=COMPAT_BUILD_SCRIPT=ok\"); }\n"
}

fn representative_main_rs() -> &'static str {
    r#"use compat_macro::compat_marker;

#[compat_marker]
fn main() {
    println!("representative-rail-ok");
}
"#
}

fn local_lib_manifest() -> &'static str {
    r#"[package]
name = "local-lib"
version = "0.1.0"
edition.workspace = true
"#
}

fn feature_lib_manifest() -> &'static str {
    r#"[package]
name = "feature-lib"
version = "0.1.0"
edition.workspace = true

[features]
default = []
extra = []
"#
}

fn target_helper_manifest() -> &'static str {
    r#"[package]
name = "target-helper"
version = "0.1.0"
edition.workspace = true
"#
}

fn proc_macro_manifest() -> &'static str {
    r#"[package]
name = "compat-macro"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true
"#
}

fn proc_macro_lib_rs() -> &'static str {
    r#"extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn compat_marker(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
"#
}

fn vendor_manifest() -> &'static str {
    r#"[package]
name = "serde"
version = "1.0.0"
edition = "2021"
"#
}

fn git_helper_manifest() -> &'static str {
    r#"[package]
name = "git-helper"
version = "0.1.0"
edition = "2021"
"#
}

fn compat_tools_manifest() -> &'static str {
    r#"[package]
name = "compat-tools"
version = "0.1.0"
edition.workspace = true

[[bin]]
name = "compat-tool"
path = "src/main.rs"
"#
}

fn cargo_config() -> &'static str {
    r#"[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"

[net]
offline = true
"#
}

fn rail_blockers(root: &Path, expected_lock_digest: &str) -> BTreeSet<&'static str> {
    let mut blockers = BTreeSet::new();
    if !root.join("vendor/serde/Cargo.toml").is_file() {
        blockers.insert(BLOCKER_MISSING_VENDOR);
    }
    let lock_digest = std::fs::read(root.join("Cargo.lock"))
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .unwrap_or_default();
    if lock_digest != expected_lock_digest {
        blockers.insert(BLOCKER_STALE_LOCK);
    }
    let build_rs = std::fs::read_to_string(root.join("build.rs")).unwrap_or_default();
    if !build_rs.contains("cargo:rustc-env=COMPAT_BUILD_SCRIPT=ok") {
        blockers.insert(BLOCKER_BUILD_METADATA);
    }
    if build_rs.contains("pkg_config::") {
        blockers.insert(BLOCKER_PKG_CONFIG);
    }
    if build_rs.contains("cargo:rustc-link-lib=") {
        blockers.insert(BLOCKER_LINK_METADATA);
    }
    if build_rs.contains("cc::Build") {
        blockers.insert(BLOCKER_NATIVE_C);
    }
    if !root.join("compat-macro/src/lib.rs").is_file() {
        blockers.insert(BLOCKER_PROC_MACRO);
    }
    if !root.join("feature-lib/Cargo.toml").is_file() {
        blockers.insert(BLOCKER_FEATURE_SURFACE);
    }
    if !root.join("target-helper/Cargo.toml").is_file() {
        blockers.insert(BLOCKER_TARGET_CFG_SURFACE);
    }
    if !std::fs::read_to_string(root.join("local-lib/Cargo.toml"))
        .unwrap_or_default()
        .contains("edition.workspace = true")
    {
        blockers.insert(BLOCKER_WORKSPACE_INHERITANCE);
    }
    if !root.join("compat-tools/src/main.rs").is_file() {
        blockers.insert(BLOCKER_MULTI_PACKAGE_BINARY);
    }
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    if manifest.contains("links = ") {
        blockers.insert(BLOCKER_NATIVE_LINK);
    }
    if manifest.contains("git = ") {
        blockers.insert(BLOCKER_VENDORED_GIT);
    }
    blockers
}

fn fake_cargo_project_ncl() -> &'static str {
    r#"let mantle = import "lib.ncl" in
let fakeSource = {
  name = "compat-demo-src",
  builder = "/bin/sh",
  args = ["-c", m%"
    BB=/bin/busybox
    $BB mkdir -p $out/src $out/local-lib/src $out/feature-lib/src $out/target-helper/src $out/compat-macro/src $out/compat-tools/src $out/vendor/serde/src $out/vendor/git-helper/src $out/.cargo
    $BB cat > $out/Cargo.toml << 'EOF'
[workspace]
members = ["local-lib", "feature-lib", "target-helper", "compat-macro", "compat-tools"]
resolver = "2"

[workspace.package]
edition = "2021"

[package]
name = "compat-demo"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[[bin]]
name = "compat-demo"
path = "src/main.rs"

[features]
default = ["feature-lib/extra"]
matrix-feature = ["feature-lib/extra"]

[dependencies]
local-lib = { path = "local-lib" }
feature-lib = { path = "feature-lib", optional = true, default-features = false, features = ["extra"] }
compat-macro = { path = "compat-macro" }
serde = "1.0.0"

[target.'cfg(unix)'.dependencies]
target-helper = { path = "target-helper" }
EOF
    $BB cat > $out/Cargo.lock << 'EOF'
# This file is automatically @generated by Cargo.
version = 4

[[package]]
name = "compat-demo"
version = "0.1.0"
dependencies = ["compat-macro", "feature-lib", "local-lib", "serde", "target-helper"]

[[package]]
name = "compat-macro"
version = "0.1.0"

[[package]]
name = "compat-tools"
version = "0.1.0"

[[package]]
name = "feature-lib"
version = "0.1.0"

[[package]]
name = "local-lib"
version = "0.1.0"

[[package]]
name = "target-helper"
version = "0.1.0"

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
EOF
    $BB cat > $out/build.rs << 'EOF'
fn main() { println!("cargo:rustc-env=COMPAT_BUILD_SCRIPT=ok"); }
EOF
    $BB cat > $out/src/main.rs << 'EOF'
use compat_macro::compat_marker;
#[compat_marker]
fn main() { println!("representative-rail-ok"); }
EOF
    $BB cat > $out/local-lib/Cargo.toml << 'EOF'
[package]
name = "local-lib"
version = "0.1.0"
edition.workspace = true
EOF
    $BB cat > $out/local-lib/src/lib.rs << 'EOF'
pub fn message() -> &'static str { "local" }
EOF
    $BB cat > $out/feature-lib/Cargo.toml << 'EOF'
[package]
name = "feature-lib"
version = "0.1.0"
edition.workspace = true

[features]
default = []
extra = []
EOF
    $BB cat > $out/feature-lib/src/lib.rs << 'EOF'
pub fn feature_enabled() -> bool { true }
EOF
    $BB cat > $out/target-helper/Cargo.toml << 'EOF'
[package]
name = "target-helper"
version = "0.1.0"
edition.workspace = true
EOF
    $BB cat > $out/target-helper/src/lib.rs << 'EOF'
pub fn target_marker() -> &'static str { "unix" }
EOF
    $BB cat > $out/compat-macro/Cargo.toml << 'EOF'
[package]
name = "compat-macro"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true
EOF
    $BB cat > $out/compat-macro/src/lib.rs << 'EOF'
extern crate proc_macro;
use proc_macro::TokenStream;
#[proc_macro_attribute]
pub fn compat_marker(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
EOF
    $BB cat > $out/compat-tools/Cargo.toml << 'EOF'
[package]
name = "compat-tools"
version = "0.1.0"
edition.workspace = true

[[bin]]
name = "compat-tool"
path = "src/main.rs"
EOF
    $BB cat > $out/compat-tools/src/main.rs << 'EOF'
fn main() { println!("compat-tool"); }
EOF
    $BB cat > $out/vendor/serde/Cargo.toml << 'EOF'
[package]
name = "serde"
version = "1.0.0"
edition = "2021"
EOF
    $BB cat > $out/vendor/serde/src/lib.rs << 'EOF'
pub fn marker() {}
EOF
    $BB cat > $out/vendor/git-helper/Cargo.toml << 'EOF'
[package]
name = "git-helper"
version = "0.1.0"
edition = "2021"
EOF
    $BB cat > $out/vendor/git-helper/src/lib.rs << 'EOF'
pub fn git_marker() {}
EOF
    $BB cat > $out/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"

[net]
offline = true
EOF
  "%],
  addressing_mode = 'input-addressed,
} | mantle.Derivation in
let fakeRust = {
  name = "rust",
  builder = "/bin/sh",
  args = ["-c", m%"
    BB=/bin/busybox
    $BB mkdir -p $out/bin
    $BB cat > $out/bin/cargo << 'EOF'
#!/bin/sh
set -eu
test -f Cargo.lock
test -f build.rs
test -f local-lib/src/lib.rs
test -f feature-lib/Cargo.toml
test -f target-helper/Cargo.toml
test -f compat-macro/src/lib.rs
test -f compat-tools/src/main.rs
test -f vendor/serde/Cargo.toml
test -f vendor/git-helper/Cargo.toml
grep 'cargo:rustc-env=COMPAT_BUILD_SCRIPT=ok' build.rs >/dev/null
grep 'feature-lib/extra' Cargo.toml >/dev/null
grep "target.'cfg(unix)'.dependencies" Cargo.toml >/dev/null
grep 'edition.workspace = true' local-lib/Cargo.toml >/dev/null
if [ "${CARGO_HOME:-}" != "/tmp/cargo-home" ]; then exit 1; fi
if [ "${CARGO_TARGET_DIR:-}" != "/tmp/cargo-target" ]; then exit 1; fi
mkdir -p "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release"
cat > "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/compat-demo" << 'BIN'
#!/bin/sh
echo representative-rail-ok
BIN
chmod +x "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/compat-demo"
EOF
    $BB chmod +x $out/bin/cargo
    $BB cat > $out/bin/rustc << 'EOF'
#!/bin/sh
echo fake rustc
EOF
    $BB chmod +x $out/bin/rustc
  "%],
  addressing_mode = 'input-addressed,
} | mantle.Derivation in
let fakeSeed = {
  name = "musl-seed-toolchain",
  builder = "/bin/sh",
  args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin $out/x86_64-linux-musl/lib; echo '#!/bin/sh' > $out/bin/x86_64-linux-musl-gcc; echo 'exit 0' >> $out/bin/x86_64-linux-musl-gcc; $BB chmod +x $out/bin/x86_64-linux-musl-gcc"],
  addressing_mode = 'input-addressed,
} | mantle.Derivation in
let fakeMusl = {
  name = "musl",
  builder = "/bin/sh",
  args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/lib; : > $out/lib/libc.so"],
  addressing_mode = 'input-addressed,
} | mantle.Derivation in
{
  packages."compat-demo" = mantle.offlineCargoPackage {
    name = "compat-demo",
    src = fakeSource,
    source_name = "compat-demo-src",
    rust = fakeRust,
    rust_name = "rust",
    seed_toolchain = fakeSeed,
    seed_toolchain_name = "musl-seed-toolchain",
    musl = fakeMusl,
    musl_name = "musl",
    binary = "compat-demo",
    target = "x86_64-unknown-linux-musl",
  },
  default.package = "compat-demo",
} | mantle.Project
"#
}

fn write_native_supported_workspace(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("local-lib/src")).unwrap();
    std::fs::write(root.join("Cargo.toml"), native_supported_manifest()).unwrap();
    std::fs::write(root.join("Cargo.lock"), native_supported_lockfile()).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() { println!(\"{}\", local_lib::value()); }\n").unwrap();
    std::fs::write(root.join("local-lib/Cargo.toml"), native_supported_local_lib_manifest()).unwrap();
    std::fs::write(root.join("local-lib/src/lib.rs"), "pub fn value() -> u32 { 42 }\n").unwrap();
}

fn native_supported_manifest() -> &'static str {
    r#"[workspace]
members = ["local-lib"]
resolver = "2"

[workspace.package]
edition = "2021"

[package]
name = "simple-compat"
version = "0.1.0"
edition.workspace = true

[[bin]]
name = "simple-compat"
path = "src/main.rs"

[dependencies]
local-lib = { path = "local-lib" }
"#
}

fn native_supported_lockfile() -> &'static str {
    r#"# This file is automatically @generated by Cargo.
version = 4

[[package]]
name = "local-lib"
version = "0.1.0"

[[package]]
name = "simple-compat"
version = "0.1.0"
dependencies = ["local-lib"]
"#
}

fn native_supported_local_lib_manifest() -> &'static str {
    r#"[package]
name = "local-lib"
version = "0.1.0"
edition.workspace = true
"#
}

fn json_array_contains(value: &Value, expected: &str) -> bool {
    value.as_array().is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn assert_matrix_binding(cargo_mode: &Value, expected_status: &str, expected_class: &str) {
    let matrix = cargo_mode.get("compatibility_surface_matrix").expect("cargo mode should include matrix binding");
    assert_eq!(matrix["matrix_id"], MATRIX_ID);
    assert_eq!(matrix["matrix_path"], MATRIX_PATH);
    assert_eq!(matrix["status"], expected_status);
    assert_eq!(matrix["evidence_class"], expected_class);
    assert!(json_array_contains(&matrix["non_claims"], "not-full-cargo-compatibility"));
}

#[test]
fn representative_fixture_contains_required_practical_rust_surfaces() {
    let fixture = tempfile::tempdir().unwrap();
    write_representative_workspace(fixture.path());

    assert!(fixture.path().join("src/main.rs").is_file());
    assert!(fixture.path().join("local-lib/src/lib.rs").is_file());
    assert!(fixture.path().join("feature-lib/Cargo.toml").is_file());
    assert!(fixture.path().join("target-helper/Cargo.toml").is_file());
    assert!(fixture.path().join("compat-macro/src/lib.rs").is_file());
    assert!(fixture.path().join("compat-tools/src/main.rs").is_file());
    assert!(fixture.path().join("build.rs").is_file());
    assert!(fixture.path().join("vendor/serde/Cargo.toml").is_file());
    assert!(fixture.path().join("vendor/git-helper/Cargo.toml").is_file());
    assert!(fixture.path().join("Cargo.lock").is_file());
    assert!(std::fs::read_to_string(fixture.path().join("Cargo.toml")).unwrap().contains("matrix-feature"));
    assert!(
        std::fs::read_to_string(fixture.path().join("local-lib/Cargo.toml"))
            .unwrap()
            .contains("edition.workspace = true")
    );
    assert!(
        std::fs::read_to_string(fixture.path().join(".cargo/config.toml"))
            .unwrap()
            .contains("offline = true")
    );
}

#[test]
fn representative_negative_fixtures_report_stable_blockers() {
    let fixture = tempfile::tempdir().unwrap();
    write_representative_workspace(fixture.path());
    let expected_lock_digest =
        blake3::hash(&std::fs::read(fixture.path().join("Cargo.lock")).unwrap()).to_hex().to_string();

    std::fs::remove_file(fixture.path().join("vendor/serde/Cargo.toml")).unwrap();
    std::fs::write(fixture.path().join("Cargo.lock"), "stale lock\n").unwrap();
    std::fs::write(
        fixture.path().join("build.rs"),
        "fn main() { println!(\"cargo:rustc-link-lib=static=crypto\"); let _ = \"pkg_config::Config\"; let _ = \"cc::Build\"; }\n",
    )
    .unwrap();
    std::fs::remove_file(fixture.path().join("compat-macro/src/lib.rs")).unwrap();
    std::fs::remove_file(fixture.path().join("feature-lib/Cargo.toml")).unwrap();
    std::fs::remove_file(fixture.path().join("target-helper/Cargo.toml")).unwrap();
    std::fs::write(
        fixture.path().join("local-lib/Cargo.toml"),
        local_lib_manifest().replace("edition.workspace = true", "edition = \"2021\""),
    )
    .unwrap();
    std::fs::remove_file(fixture.path().join("compat-tools/src/main.rs")).unwrap();
    std::fs::write(
        fixture.path().join("Cargo.toml"),
        representative_manifest()
            .replace("build = \"build.rs\"", "build = \"build.rs\"\nlinks = \"native\"")
            .replace("serde = \"1.0.0\"", "serde = \"1.0.0\"\ngit-helper = { git = \"https://example.invalid/git-helper\", rev = \"0000000000000000000000000000000000000000\" }"),
    )
    .unwrap();

    let blockers = rail_blockers(fixture.path(), &expected_lock_digest);

    assert!(blockers.contains(BLOCKER_MISSING_VENDOR), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_STALE_LOCK), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_BUILD_METADATA), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_PROC_MACRO), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_NATIVE_LINK), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_FEATURE_SURFACE), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_TARGET_CFG_SURFACE), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_WORKSPACE_INHERITANCE), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_MULTI_PACKAGE_BINARY), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_VENDORED_GIT), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_PKG_CONFIG), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_LINK_METADATA), "blockers: {blockers:?}");
    assert!(blockers.contains(BLOCKER_NATIVE_C), "blockers: {blockers:?}");
}

#[test]
fn representative_offline_cargo_rail_runs_sandbox_smoke_without_network_inputs() {
    if !can_build() {
        eprintln!("{BWRAP_SKIP}");
        return;
    }
    let project = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("mantle-project.ncl"), fake_cargo_project_ncl()).unwrap();

    let output = mantle_cmd()
        .current_dir(project.path())
        .arg("--store")
        .arg(store.path())
        .arg("--state-dir")
        .arg(state.path())
        .arg("run")
        .arg(".#compat-demo")
        .arg("--no-substitute")
        .output()
        .expect("mantle run should execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert_eq!(stdout.trim(), EXPECTED_STDOUT);
    assert!(stderr.contains("running:"), "stderr should identify the selected binary: {stderr}");
}

#[test]
fn native_path_only_surface_reports_matrix_binding_without_cargo_fallback() {
    let fixture = tempfile::tempdir().unwrap();
    let shim_dir = tempfile::tempdir().unwrap();
    write_native_supported_workspace(fixture.path());
    let shim = shim_dir.path().join("cargo");
    std::fs::write(&shim, FAILING_CARGO_SHIM).unwrap();
    make_executable(&shim);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(fixture.path())
        .arg("--cargo")
        .arg(&shim)
        .arg("--no-cargo-oracle")
        .output()
        .expect("mantle rust-plan should execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(!stderr.contains("cargo must not be invoked"), "rust-plan fell back to Cargo: {stderr}");
    let receipt: Value = serde_json::from_str(&stdout).expect("rust-plan JSON receipt");
    let cargo_mode = receipt.get("cargo_mode").expect("cargo_mode exists");
    assert_eq!(cargo_mode["compatibility_class"], "cargo-free-bounded-topology");
    assert_matrix_binding(cargo_mode, "supported", "cargo-free-bounded-topology");
    let surface_ids = &cargo_mode["compatibility_surface_matrix"]["surface_ids"];
    assert!(json_array_contains(surface_ids, "path-workspace-basic"), "{surface_ids:#?}");
    assert!(json_array_contains(surface_ids, "local-path-dependency"), "{surface_ids:#?}");
    assert!(json_array_contains(surface_ids, "workspace-inheritance"), "{surface_ids:#?}");
    assert!(json_array_contains(surface_ids, "source-closure-digest"), "{surface_ids:#?}");
    assert!(json_array_contains(surface_ids, "unit-graph-facts"), "{surface_ids:#?}");
    let digest = receipt
        .pointer("/source_closure/sources/0/source_digest/value")
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert_eq!(digest.len(), BLAKE3_HEX_BYTES, "source digest should be BLAKE3-bound: {stdout}");
    assert_eq!(receipt["native_unit_graph_planning"]["ready"], true, "unit graph should be ready: {stdout}");
}

#[test]
fn representative_rust_plan_receipt_is_bounded_success_or_blocker_without_cargo_fallback() {
    let fixture = tempfile::tempdir().unwrap();
    let shim_dir = tempfile::tempdir().unwrap();
    write_representative_workspace(fixture.path());
    let shim = shim_dir.path().join("cargo");
    std::fs::write(&shim, FAILING_CARGO_SHIM).unwrap();
    make_executable(&shim);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(fixture.path())
        .arg("--cargo")
        .arg(&shim)
        .arg("--no-cargo-oracle")
        .output()
        .expect("mantle rust-plan should execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(!stderr.contains("cargo must not be invoked"), "rust-plan fell back to Cargo: {stderr}");
    let receipt: Value = serde_json::from_str(&stdout).expect("rust-plan JSON receipt");
    let cargo_mode = receipt.get("cargo_mode").expect("cargo_mode exists");
    let class = cargo_mode["compatibility_class"].as_str().unwrap_or_default();
    assert!(
        class == "cargo-free-bounded-topology" || class == "blocked-unsupported-surface",
        "unexpected compatibility class `{class}` in {stdout}"
    );
    let expected_status = if class == "cargo-free-bounded-topology" {
        "supported"
    } else {
        "blocked"
    };
    assert_matrix_binding(cargo_mode, expected_status, class);
    let matrix = &cargo_mode["compatibility_surface_matrix"];
    if expected_status == "blocked" {
        assert!(json_array_contains(&matrix["surface_ids"], "blocked-unsupported-surface"), "{matrix:#?}");
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_mode(EXECUTABLE_MODE);
    std::fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
