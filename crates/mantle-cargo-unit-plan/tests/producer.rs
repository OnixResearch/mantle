use std::fs;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde_json::Value;
use serde_json::json;

fn tool(name: &str) -> PathBuf {
    let explicit = std::env::var_os(name.to_ascii_uppercase());
    let ambient_path = std::env::var_os("PATH").unwrap();
    let paths = explicit
        .into_iter()
        .map(PathBuf::from)
        .chain(std::env::split_paths(&ambient_path).map(|directory| directory.join(name)));
    for path in paths {
        if path.is_file() {
            return fs::canonicalize(path).unwrap();
        }
    }
    panic!("missing declared test tool {name}");
}

fn fixture_root() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "mantle-cargo-unit-producer-registry-{}-{unique}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    directory
}

#[test]
fn unavailable_registry_source_cannot_use_network_or_ambient_cargo_home() {
    let fixture = fixture_root();
    let store = fixture.join("store");
    let workspace = store.join("fedcba9876543210fedcba9876543210-workspace");
    let tools = store.join("0123456789abcdef0123456789abcdef-toolchain");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(tools.join("bin")).unwrap();
    symlink(tool("cargo"), tools.join("bin/cargo")).unwrap();
    symlink(tool("rustc"), tools.join("bin/rustc")).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    fs::create_dir(workspace.join(".cargo")).unwrap();
    fs::write(
        workspace.join(".cargo/config.toml"),
        format!(
            "[registries.crates-io]\nindex = 'sparse+http://127.0.0.1:{port}/index/'\n[net]\nretry = 0\n[http]\ntimeout = 1\n"
        ),
    )
    .unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = 'offline-probe'\nversion = '0.1.0'\nedition = '2024'\n[dependencies]\nmantle_unit_absent_460372 = '=0.1.0'\n",
    )
    .unwrap();
    fs::write(
        workspace.join("Cargo.lock"),
        "version = 4\n\n[[package]]\nname = 'offline-probe'\nversion = '0.1.0'\ndependencies = ['mantle_unit_absent_460372']\n\n[[package]]\nname = 'mantle_unit_absent_460372'\nversion = '0.1.0'\nsource = 'registry+https://github.com/rust-lang/crates.io-index'\nchecksum = '0000000000000000000000000000000000000000000000000000000000000000'\n",
    )
    .unwrap();
    fs::write(workspace.join("src/main.rs"), "fn main() {}\n").unwrap();

    let config = json!({
        "workspace": workspace.display().to_string(),
        "vendor": null,
        "planner": tools.join("bin/mantle").display().to_string(),
        "cargo": tools.join("bin/cargo").display().to_string(),
        "rustc": tools.join("bin/rustc").display().to_string(),
        "linker": tools.join("bin/cc").display().to_string(),
        "toolchain": tools.display().to_string(),
        "helper": tools.join("bin/mantle-cargo-unit-helper").display().to_string(),
        "runtime_inputs": [],
        "store_prefix": store.display().to_string(),
        "system": "x86_64-linux",
        "host_triple": "x86_64-unknown-linux-gnu",
        "target_triple": "x86_64-unknown-linux-gnu",
        "profile": "dev"
    });
    let config_path = fixture.join("config.json");
    fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
    let ambient_home = fixture.join("ambient-cargo-home");
    fs::create_dir(&ambient_home).unwrap();
    fs::write(
        ambient_home.join("config.toml"),
        format!(
            "[source.crates-io]\nreplace-with = 'ambient-vendor'\n[source.ambient-vendor]\ndirectory = {:?}\n[net]\noffline = false\n",
            fixture.join("ambient-vendor-absent")
        ),
    )
    .unwrap();
    let plan = fixture.join("plan");
    let output = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-producer"))
        .env("MANTLE_UNIT_CONFIG", &config_path)
        .env("MANTLE_UNIT_RUNTIME_INPUTS", "[]")
        .env("TMPDIR", &fixture)
        .env("sources", fixture.join("sources"))
        .env("plan", &plan)
        .env("out", fixture.join("out"))
        .env("CARGO_HOME", &ambient_home)
        .env("CARGO_NET_OFFLINE", "false")
        .env("CARGO_REGISTRIES_CRATES_IO_INDEX", format!("sparse+http://127.0.0.1:{port}/index/"))
        .env("RUSTFLAGS", "-C link-arg=--ambient")
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "producer unexpectedly accepted unavailable registry package: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let blockers: Vec<Value> =
        serde_json::from_slice(&fs::read(fixture.join("out/cargo-unit-plan-blockers.json")).unwrap()).unwrap();
    assert!(
        blockers.iter().any(|blocker| {
            blocker["code"] == "unit-plan-oracle-failure"
                && blocker["subject"] == "cargo metadata"
                && blocker["detail"].as_str().unwrap().contains("mantle_unit_absent_460372")
        }),
        "wrong blocker: {blockers:?}"
    );
    assert!(!plan.exists(), "blocked producer emitted a plan");
    assert!(!fixture.join("out/cargo-unit-plan-evidence.json").exists());
    assert!(matches!(listener.accept(), Err(error) if error.kind() == ErrorKind::WouldBlock));
    let mut changed_config = config;
    changed_config["runtime_inputs"] = json!([tools.display().to_string()]);
    fs::write(&config_path, serde_json::to_vec(&changed_config).unwrap()).unwrap();
    let rejected = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-producer"))
        .env("MANTLE_UNIT_CONFIG", &config_path)
        .env("MANTLE_UNIT_RUNTIME_INPUTS", "[]")
        .env("out", fixture.join("runtime-mismatch"))
        .output()
        .unwrap();
    assert!(!rejected.status.success(), "undeclared runtime loader was accepted");
    let runtime_blockers: Vec<Value> =
        serde_json::from_slice(&fs::read(fixture.join("runtime-mismatch/cargo-unit-plan-blockers.json")).unwrap())
            .unwrap();
    assert!(
        runtime_blockers
            .iter()
            .any(|blocker| { blocker["code"] == "unit-plan-host-path" && blocker["subject"] == "runtime_inputs" }),
        "{runtime_blockers:?}"
    );
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn missing_native_role_rejects_before_copying_workspace_or_starting_cargo() {
    let fixture = fixture_root();
    let store = fixture.join("store");
    let existing_native = store.join("0123456789abcdef0123456789abcdef-native-library");
    fs::create_dir_all(&existing_native).unwrap();
    let package_id = "path+file:///build/workspace/native-answer-sys#0.1.0";
    let declared_native = json!({
        (package_id): {
            "inputs": {"native": existing_native.display().to_string()},
            "env": {"NATIVE_ANSWER_ROOT": "{{role:missing}}"},
            "path": []
        }
    });
    let input = store.join("fedcba9876543210fedcba9876543210-unavailable-workspace");
    let mut config = json!({
        "workspace": input.display().to_string(),
        "vendor": null,
        "planner": input.display().to_string(),
        "cargo": input.display().to_string(),
        "rustc": input.display().to_string(),
        "linker": input.display().to_string(),
        "toolchain": input.display().to_string(),
        "helper": input.display().to_string(),
        "runtime_inputs": [],
        "store_prefix": store.display().to_string(),
        "system": "x86_64-linux",
        "host_triple": "x86_64-unknown-linux-gnu",
        "target_triple": "x86_64-unknown-linux-gnu",
        "profile": "dev",
        "native_inputs": declared_native
    });
    let config_path = fixture.join("config.json");
    fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-producer"))
        .env("MANTLE_UNIT_CONFIG", &config_path)
        .env("MANTLE_UNIT_NATIVE_INPUTS", serde_json::to_string(&declared_native).unwrap())
        .env("MANTLE_UNIT_RUNTIME_INPUTS", "[]")
        .env("TMPDIR", &fixture)
        .env("out", fixture.join("out"))
        .output()
        .unwrap();
    assert!(!output.status.success(), "undeclared role accepted");
    let blockers: Vec<Value> =
        serde_json::from_slice(&fs::read(fixture.join("out/cargo-unit-plan-blockers.json")).unwrap()).unwrap();
    assert_eq!(blockers[0]["code"], "native-input-role-undeclared", "{blockers:?}");
    assert_eq!(blockers[0]["subject"], package_id);
    assert!(!fixture.join("cargo-unit-plan-workspace").exists(), "workspace copy or Cargo oracle started");
    assert!(existing_native.exists(), "declared library disappeared");
    config["native_inputs"][package_id]["env"]["NATIVE_ANSWER_ROOT"] = json!("{{role:native}}");
    fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
    let valid = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-producer"))
        .env("MANTLE_UNIT_CONFIG", &config_path)
        .env("MANTLE_UNIT_NATIVE_INPUTS", serde_json::to_string(&config["native_inputs"]).unwrap())
        .env("MANTLE_UNIT_RUNTIME_INPUTS", "[]")
        .env("TMPDIR", &fixture)
        .env("out", fixture.join("valid-role-out"))
        .output()
        .unwrap();
    assert!(!valid.status.success(), "unavailable workspace unexpectedly copied");
    let valid_blockers: Vec<Value> =
        serde_json::from_slice(&fs::read(fixture.join("valid-role-out/cargo-unit-plan-blockers.json")).unwrap())
            .unwrap();
    assert_eq!(valid_blockers[0]["code"], "unit-plan-source-unsupported", "{valid_blockers:?}");
    assert!(fixture.join("cargo-unit-plan-workspace").exists(), "valid native role did not pass preflight");
    fs::remove_dir_all(fixture).unwrap();
}
