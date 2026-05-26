use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

#[test]
fn rust_plan_cli_reports_native_host_unit_graph_for_build_script() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("native-build-script");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"native-build-script\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("build.rs"), "fn main() {}\n").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .output()
        .expect("rust-plan CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let host_graph = &receipt["native_host_unit_graph_planning"];
    assert!(host_graph["ready"].as_bool().unwrap(), "{receipt:#?}");
    assert_eq!(host_graph["comparison_status"], "matched");
    assert_eq!(host_graph["host_units"].as_array().unwrap().len(), 1);
    assert_eq!(host_graph["host_units"][0]["target_kind"], "custom-build");
    assert_eq!(host_graph["target_consumers"].as_array().unwrap().len(), 1);
    assert_eq!(host_graph["target_consumers"][0]["consumed_host_artifacts"][0]["target_kind"], "custom-build");
    assert!(receipt["unit_derivation_graph"]["ready"].as_bool().unwrap());
    assert_eq!(receipt["unit_derivation_graph"]["host_unit_count"], 1);
    assert_eq!(receipt["unit_derivation_graph"]["host_artifact_count"], 1);
    assert_eq!(receipt["unit_derivation_graph"]["derivations"].as_array().unwrap().len(), 2);
    assert_eq!(receipt["unit_derivation_graph"]["derivations"][0]["target_kind"], "lib");
    assert_eq!(
        receipt["unit_derivation_graph"]["derivations"][0]["consumed_host_artifacts"][0]["target_kind"],
        "custom-build"
    );
}

#[test]
fn rust_plan_cli_reports_native_host_unit_graph_for_proc_macro_dependency() {
    let dir = TempDir::new().unwrap();
    let macro_dir = dir.path().join("demo-macro");
    let app_dir = dir.path().join("app-crate");
    std::fs::create_dir_all(macro_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::write(
        macro_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-macro\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nproc-macro = true\n",
    )
    .unwrap();
    std::fs::write(
        macro_dir.join("src/lib.rs"),
        "extern crate proc_macro;\nuse proc_macro::TokenStream;\n#[proc_macro_attribute]\npub fn demo(_attr: TokenStream, item: TokenStream) -> TokenStream { item }\n",
    )
    .unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"app-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_macro = { package = \"demo-macro\", path = \"../demo-macro\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "use demo_macro::demo;\n#[demo]\npub fn value() -> u32 { 1 }\n")
        .unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&app_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .output()
        .expect("rust-plan CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let host_graph = &receipt["native_host_unit_graph_planning"];
    assert!(host_graph["ready"].as_bool().unwrap(), "{receipt:#?}");
    assert_eq!(host_graph["host_units"].as_array().unwrap().len(), 1);
    assert_eq!(host_graph["host_units"][0]["target_kind"], "proc-macro");
    assert_eq!(host_graph["target_consumers"].as_array().unwrap().len(), 1);
    assert_eq!(host_graph["target_consumers"][0]["target_kind"], "lib");
    assert_eq!(host_graph["target_consumers"][0]["consumed_host_artifacts"][0]["target_kind"], "proc-macro");
    assert!(receipt["unit_derivation_graph"]["ready"].as_bool().unwrap());
    assert_eq!(receipt["unit_derivation_graph"]["host_unit_count"], 1);
    assert_eq!(receipt["unit_derivation_graph"]["derivations"][1]["target_kind"], "proc-macro");
}

#[test]
fn rust_plan_cli_blocks_native_host_unit_graph_when_native_fragment_is_unsupported() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("feature-build-script");
    let helper_dir = dir.path().join("helper-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::create_dir_all(helper_dir.join("src")).unwrap();
    std::fs::write(
        helper_dir.join("Cargo.toml"),
        "[package]\nname = \"helper-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nunsupported = []\n",
    )
    .unwrap();
    std::fs::write(helper_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"feature-build-script\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n\n[build-dependencies]\nhelper_crate = { package = \"helper-crate\", path = \"../helper-crate\", features = [\"unsupported\"] }\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("build.rs"), "fn main() {}\n").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .output()
        .expect("rust-plan CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let host_graph = &receipt["native_host_unit_graph_planning"];
    assert!(!host_graph["ready"].as_bool().unwrap(), "{receipt:#?}");
    assert_eq!(host_graph["blockers"][0]["class"], "native-package-target-planning-blocked");
    assert!(
        receipt["native_package_target_planning"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker["class"] == "unsupported-build-dependency-options"),
        "{receipt:#?}"
    );
}

#[test]
fn rust_plan_cli_blocks_host_artifact_topology_when_native_host_graph_is_not_ready() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("feature-build-script-exec");
    let helper_dir = dir.path().join("helper-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::create_dir_all(helper_dir.join("src")).unwrap();
    std::fs::write(
        helper_dir.join("Cargo.toml"),
        "[package]\nname = \"helper-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nunsupported = []\n",
    )
    .unwrap();
    std::fs::write(helper_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"feature-build-script-exec\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n\n[build-dependencies]\nhelper_crate = { package = \"helper-crate\", path = \"../helper-crate\", features = [\"unsupported\"] }\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("build.rs"), "fn main() {}\n").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("native-blocked-host-output"))
        .output()
        .expect("rust-plan host-artifact topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(!receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap());
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["blocker"]["class"], "native-host-unit-graph-blocked");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_blocks_unified_topology_when_native_host_graph_is_not_ready() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("feature-build-script-unified");
    let helper_dir = dir.path().join("helper-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::create_dir_all(helper_dir.join("src")).unwrap();
    std::fs::write(
        helper_dir.join("Cargo.toml"),
        "[package]\nname = \"helper-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nunsupported = []\n",
    )
    .unwrap();
    std::fs::write(helper_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"feature-build-script-unified\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n\n[build-dependencies]\nhelper_crate = { package = \"helper-crate\", path = \"../helper-crate\", features = [\"unsupported\"] }\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("build.rs"), "fn main() {}\n").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("native-blocked-unified-output"))
        .output()
        .expect("rust-plan unified topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(!receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap());
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["blocker"]["class"], "native-host-unit-graph-blocked");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_supported_unit_from_explicit_receipt_material() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("cli-exec");
    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"cli-exec\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"cli-exec\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn answer() -> u32 { 42 }\n").unwrap();

    let output_root = dir.path().join("unit-output");
    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-first-supported-unit")
        .arg("--execution-output-root")
        .arg(&output_root)
        .output()
        .expect("rust-plan execution CLI should run");

    assert!(
        output.status.success(),
        "rust-plan execution CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let execution = &receipt["unit_execution"];
    assert_eq!(execution["execution_status"], "success");
    assert_eq!(execution["rebuild_reason"], "rebuilt-explicit-unit");
    assert_eq!(execution["target_kind"], "lib");
    assert_eq!(execution["toolchain"]["tool"], "rustc");
    assert!(execution["blocker"].is_null());
    assert!(execution["output_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().starts_with("declared-output/")
            && artifact["path"].as_str().unwrap().ends_with(".rlib")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
    assert!(receipt["rust_plan"]["unit_derivation_graph"]["ready"].as_bool().unwrap());
    assert_eq!(receipt["rust_plan"]["unit_derivation_graph"]["derivation_count"], 1);
}

#[test]
fn rust_plan_cli_executes_dependency_chain_from_explicit_receipt_material() {
    let dir = TempDir::new().unwrap();
    let dep_dir = dir.path().join("dep-crate");
    let app_dir = dir.path().join("app-crate");
    std::fs::create_dir_all(dep_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::write(
        dep_dir.join("Cargo.toml"),
        "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dep_dir.join("src/lib.rs"), "pub fn answer() -> u32 { 42 }\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"app-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", path = \"../dep-crate\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn call_dep() -> u32 { dep_crate::answer() }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&app_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(
        lock_output.status.success(),
        "cargo generate-lockfile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&lock_output.stdout),
        String::from_utf8_lossy(&lock_output.stderr)
    );

    let output_root = dir.path().join("chain-output");
    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-first-dependency-chain")
        .arg("--execution-output-root")
        .arg(&output_root)
        .output()
        .expect("rust-plan dependency-chain CLI should run");

    assert!(
        output.status.success(),
        "rust-plan dependency-chain CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["unit_derivation_graph"]["ready"].as_bool().unwrap());
    let chain = &receipt["dependency_chain_execution"];
    assert_eq!(chain["execution_status"], "success", "{receipt:#?}");
    assert!(chain["claim"].as_str().unwrap().contains("bounded explicit Rust dependency edge"));
    assert!(chain["blocker"].is_null());
    let unit_executions = chain["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2);
    assert_eq!(unit_executions[0]["target_kind"], "lib");
    assert_eq!(unit_executions[1]["target_kind"], "lib");
    assert_eq!(unit_executions[0]["execution_status"], "success");
    assert_eq!(unit_executions[1]["execution_status"], "success");
    assert!(
        unit_executions[1]["dependency_artifact_digests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|artifact| artifact["blake3"].as_str().unwrap().len() == 64)
    );
    assert!(unit_executions.iter().all(|execution| {
        execution["output_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
            artifact["path"].as_str().unwrap().starts_with("declared-output/")
                && artifact["blake3"].as_str().unwrap().len() == 64
        })
    }));
}

#[test]
fn rust_plan_cli_executes_target_unit_topology_from_explicit_receipt_material() {
    let dir = TempDir::new().unwrap();
    let dep_a_dir = dir.path().join("dep-a");
    let dep_b_dir = dir.path().join("dep-b");
    let app_dir = dir.path().join("app-crate");
    for crate_dir in [&dep_a_dir, &dep_b_dir, &app_dir] {
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    }
    std::fs::write(
        dep_a_dir.join("Cargo.toml"),
        "[package]\nname = \"dep-a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dep_a_dir.join("src/lib.rs"), "pub fn a() -> u32 { 40 }\n").unwrap();
    std::fs::write(
        dep_b_dir.join("Cargo.toml"),
        "[package]\nname = \"dep-b\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_a = { package = \"dep-a\", path = \"../dep-a\" }\n",
    )
    .unwrap();
    std::fs::write(dep_b_dir.join("src/lib.rs"), "pub fn b() -> u32 { dep_a::a() + 1 }\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"app-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_b = { package = \"dep-b\", path = \"../dep-b\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn call_dep() -> u32 { dep_b::b() + 1 }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&app_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(
        lock_output.status.success(),
        "cargo generate-lockfile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&lock_output.stdout),
        String::from_utf8_lossy(&lock_output.stderr)
    );

    let output_root = dir.path().join("topology-output");
    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-target-topology")
        .arg("--execution-output-root")
        .arg(&output_root)
        .output()
        .expect("rust-plan target-topology CLI should run");

    assert!(
        output.status.success(),
        "rust-plan target-topology CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["unit_derivation_graph"]["ready"].as_bool().unwrap());
    let topology = &receipt["target_topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    assert!(topology["claim"].as_str().unwrap().contains("bounded target-only Rust unit topology"));
    assert!(topology["blocker"].is_null());
    assert_eq!(topology["receipt_hash"].as_str().unwrap().len(), 64);
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 3);
    assert!(unit_executions.iter().all(|execution| execution["execution_status"] == "success"));
    assert!(
        unit_executions[1]["dependency_artifact_digests"].as_array().unwrap()[0]["blake3"]
            .as_str()
            .unwrap()
            .len()
            == 64
    );
    assert!(
        unit_executions[2]["dependency_artifact_digests"].as_array().unwrap()[0]["blake3"]
            .as_str()
            .unwrap()
            .len()
            == 64
    );
}

#[test]
fn rust_plan_cli_blocks_target_topology_with_proc_macro_host_artifact() {
    let dir = TempDir::new().unwrap();
    let macro_dir = dir.path().join("demo-macro");
    let app_dir = dir.path().join("app-crate");
    std::fs::create_dir_all(macro_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::write(
        macro_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-macro\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nproc-macro = true\n",
    )
    .unwrap();
    std::fs::write(
        macro_dir.join("src/lib.rs"),
        "extern crate proc_macro;\nuse proc_macro::TokenStream;\n#[proc_macro_attribute]\npub fn demo(_attr: TokenStream, item: TokenStream) -> TokenStream { item }\n",
    )
    .unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"app-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_macro = { package = \"demo-macro\", path = \"../demo-macro\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "use demo_macro::demo;\n#[demo]\npub fn value() -> u32 { 1 }\n")
        .unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&app_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-target-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("blocked-output"))
        .output()
        .expect("rust-plan target-topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let topology = &receipt["target_topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["blocker"]["class"], "unsupported-topology-shape");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_proc_macro_host_artifact_topology() {
    let dir = TempDir::new().unwrap();
    let macro_dir = dir.path().join("demo-macro");
    let app_dir = dir.path().join("app-crate");
    std::fs::create_dir_all(macro_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::write(
        macro_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-macro\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nproc-macro = true\n",
    )
    .unwrap();
    std::fs::write(
        macro_dir.join("src/lib.rs"),
        "extern crate proc_macro;\nuse proc_macro::TokenStream;\n#[proc_macro_attribute]\npub fn demo(_attr: TokenStream, item: TokenStream) -> TokenStream { item }\n",
    )
    .unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"app-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_macro = { package = \"demo-macro\", path = \"../demo-macro\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "use demo_macro::demo;\n#[demo]\npub fn value() -> u32 { 1 }\n")
        .unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&app_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("host-output"))
        .output()
        .expect("rust-plan host-artifact topology CLI should run");

    assert!(
        output.status.success(),
        "rust-plan host-artifact CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["unit_derivation_graph"]["ready"].as_bool().unwrap());
    assert!(receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap());
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    assert!(topology["claim"].as_str().unwrap().contains("bounded Rust host-artifact topology"));
    assert!(topology["blocker"].is_null());
    assert_eq!(topology["receipt_hash"].as_str().unwrap().len(), 64);
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2);
    assert_eq!(unit_executions[0]["target_kind"], "proc-macro");
    assert_eq!(unit_executions[1]["target_kind"], "lib");
    assert!(unit_executions.iter().all(|execution| execution["execution_status"] == "success"));
    assert!(unit_executions[1]["host_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("demo_macro") && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_executes_build_script_metadata_topology() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("build-meta-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"build-meta-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("build.rs"),
        "fn main() {\n    let out = std::env::var(\"OUT_DIR\").unwrap();\n    std::fs::write(format!(\"{out}/generated.txt\"), \"generated-ok\").unwrap();\n    println!(\"cargo:rustc-cfg=mantle_build_script\");\n    println!(\"cargo:rustc-env=BUILD_VALUE=env-ok\");\n    println!(\"cargo:rerun-if-changed=build.rs\");\n}\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        "#[cfg(not(mantle_build_script))]\ncompile_error!(\"missing build script cfg\");\npub const BUILD_VALUE: &str = env!(\"BUILD_VALUE\");\npub const GENERATED: &str = include_str!(concat!(env!(\"OUT_DIR\"), \"/generated.txt\"));\n",
    )
    .unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("build-meta-output"))
        .output()
        .expect("rust-plan host-artifact topology CLI should run");

    assert!(
        output.status.success(),
        "rust-plan build-script metadata CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap());
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let metadata_runs = topology["build_script_metadata_runs"].as_array().unwrap();
    assert_eq!(metadata_runs.len(), 1, "{receipt:#?}");
    assert_eq!(metadata_runs[0]["rustc_cfg"].as_array().unwrap()[0], "mantle_build_script");
    assert_eq!(metadata_runs[0]["rustc_env"]["BUILD_VALUE"], "env-ok");
    assert_eq!(metadata_runs[0]["rerun_if_changed"].as_array().unwrap()[0], "build.rs");
    assert!(metadata_runs[0]["out_dir_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"] == "out-dir/generated.txt" && artifact["blake3"].as_str().unwrap().len() == 64
    }));
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2);
    assert_eq!(unit_executions[0]["target_kind"], "custom-build");
    assert_eq!(unit_executions[1]["target_kind"], "lib");
    assert!(unit_executions.iter().all(|execution| execution["execution_status"] == "success"));
}

#[test]
fn rust_plan_cli_binds_build_script_native_link_metadata() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("build-link-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"build-link-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("build.rs"),
        "fn main() {\n    println!(\"cargo:rustc-link-search=native=/tmp\");\n    println!(\"cargo:rustc-link-lib=m\");\n}\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("src/main.rs"),
        "unsafe extern \"C\" { fn cos(input: f64) -> f64; }\nfn main() { let _ = unsafe { cos(0.0) }; }\n",
    )
    .unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("build-link-output"))
        .output()
        .expect("rust-plan host-artifact topology CLI should run");

    assert!(
        output.status.success(),
        "rust-plan build-script link metadata CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap());
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let metadata_runs = topology["build_script_metadata_runs"].as_array().unwrap();
    assert_eq!(metadata_runs.len(), 1, "{receipt:#?}");
    assert_eq!(metadata_runs[0]["rustc_link_lib"].as_array().unwrap()[0], "m");
    assert_eq!(metadata_runs[0]["rustc_link_search"].as_array().unwrap()[0], "native=/tmp");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2);
    assert_eq!(unit_executions[0]["target_kind"], "custom-build");
    assert_eq!(unit_executions[1]["target_kind"], "bin");
    assert!(unit_executions.iter().all(|execution| execution["execution_status"] == "success"));
}

#[test]
fn rust_plan_cli_executes_unified_topology_with_build_script_and_bin() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("unified-topology-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"unified-topology-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("build.rs"),
        "fn main() {\n    println!(\"cargo:rustc-cfg=mantle_unified_topology\");\n    println!(\"cargo:rustc-env=UNIFIED_VALUE=unified-ok\");\n}\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        "#[cfg(not(mantle_unified_topology))]\ncompile_error!(\"missing unified topology cfg\");\npub fn value() -> &'static str { env!(\"UNIFIED_VALUE\") }\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("src/main.rs"), "fn main() { let _ = unified_topology_crate::value(); }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("unified-output"))
        .output()
        .expect("rust-plan unified topology CLI should run");

    assert!(
        output.status.success(),
        "rust-plan unified topology CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["unit_derivation_graph"]["ready"].as_bool().unwrap());
    assert!(receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap());
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    assert!(topology["claim"].as_str().unwrap().contains("bounded unified Rust unit topology"));
    assert_eq!(topology["receipt_hash"].as_str().unwrap().len(), 64);
    let metadata_runs = topology["build_script_metadata_runs"].as_array().unwrap();
    assert_eq!(metadata_runs.len(), 1, "{receipt:#?}");
    assert_eq!(metadata_runs[0]["rustc_cfg"].as_array().unwrap()[0], "mantle_unified_topology");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    let target_kinds = unit_executions
        .iter()
        .map(|execution| execution["target_kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(target_kinds, vec!["custom-build", "lib", "bin"], "{receipt:#?}");
    assert!(unit_executions.iter().all(|execution| execution["execution_status"] == "success"));
    assert!(unit_executions[2]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libunified_topology_crate")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_executes_unified_topology_for_proc_macro_shape() {
    let dir = TempDir::new().unwrap();
    let macro_dir = dir.path().join("demo-macro");
    let app_dir = dir.path().join("app-crate");
    std::fs::create_dir_all(macro_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::write(
        macro_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-macro\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nproc-macro = true\n",
    )
    .unwrap();
    std::fs::write(
        macro_dir.join("src/lib.rs"),
        "extern crate proc_macro;\nuse proc_macro::TokenStream;\n#[proc_macro_attribute]\npub fn demo(_attr: TokenStream, item: TokenStream) -> TokenStream { item }\n",
    )
    .unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"app-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_macro = { package = \"demo-macro\", path = \"../demo-macro\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "use demo_macro::demo;\n#[demo]\npub fn value() -> u32 { 1 }\n")
        .unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&app_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("unified-proc-macro-output"))
        .output()
        .expect("rust-plan unified topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap());
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    assert!(topology["blocker"].is_null());
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2);
    assert_eq!(unit_executions[0]["target_kind"], "proc-macro");
    assert_eq!(unit_executions[1]["target_kind"], "lib");
}

#[test]
fn rust_plan_cli_blocks_unsupported_build_script_link_metadata() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("bad-build-link-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"bad-build-link-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("build.rs"),
        "fn main() { println!(\"cargo:rustc-link-lib=static:+whole-archive=m\"); }\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("bad-build-link-output"))
        .output()
        .expect("rust-plan host-artifact topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["blocker"]["class"], "malformed-build-script-metadata");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 1);
    assert_eq!(topology["unit_executions"].as_array().unwrap()[0]["target_kind"], "custom-build");
}

#[test]
fn rust_plan_cli_blocks_malformed_build_script_metadata() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("bad-build-meta-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"bad-build-meta-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("build.rs"), "fn main() { println!(\"cargo:rustc-env=BROKEN\"); }\n").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
    let lock_output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&crate_dir)
        .output()
        .expect("cargo generate-lockfile should run");
    assert!(lock_output.status.success(), "{}", String::from_utf8_lossy(&lock_output.stderr));

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("bad-build-meta-output"))
        .output()
        .expect("rust-plan host-artifact topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["blocker"]["class"], "malformed-build-script-metadata");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 1);
    assert_eq!(topology["unit_executions"].as_array().unwrap()[0]["target_kind"], "custom-build");
}

#[test]
fn rust_plan_cli_blocks_host_artifact_topology_without_host_units() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("plain-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"plain-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("Cargo.lock"), "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"plain-crate\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("no-host-output"))
        .output()
        .expect("rust-plan host-artifact topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["blocker"]["class"], "missing-host-artifact-unit");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
}

fn write_vendored_registry_fixture(dir: &TempDir, unsupported_registry_bench: bool) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-topology-app");
    let vendor_dir = app_dir.join("vendor/demo-dep-0.1.0");
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join(".cargo")).unwrap();
    std::fs::create_dir_all(vendor_dir.join("src")).unwrap();
    if unsupported_registry_bench {
        std::fs::create_dir_all(vendor_dir.join("benches")).unwrap();
    }
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"registry-topology-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_dep = { package = \"demo-dep\", version = \"=0.1.0\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_dep::value() + 1 }\n").unwrap();
    std::fs::write(
        app_dir.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
    )
    .unwrap();
    let bench = if unsupported_registry_bench {
        "\n[[bench]]\nname = \"unsupported\"\npath = \"benches/unsupported.rs\"\n"
    } else {
        ""
    };
    std::fs::write(
        vendor_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"demo-dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_dep\"\npath = \"src/lib.rs\"\n{bench}"
        ),
    )
    .unwrap();
    std::fs::write(vendor_dir.join("src/lib.rs"), "pub fn value() -> u32 { 41 }\n").unwrap();
    if unsupported_registry_bench {
        std::fs::write(vendor_dir.join("benches/unsupported.rs"), "fn main() {}\n").unwrap();
    }
    std::fs::write(vendor_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"fakechecksum\"}\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"registry-topology-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-dep\",\n]\n\n[[package]]\nname = \"demo-dep\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"fakechecksum\"\n",
    )
    .unwrap();
    app_dir
}

fn write_feature_gated_vendored_registry_fixture(dir: &TempDir) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-feature-topology-app");
    let mid_dir = app_dir.join("vendor/demo-feature-mid-0.1.0");
    let leaf_dir = app_dir.join("vendor/demo-feature-leaf-0.1.0");
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join(".cargo")).unwrap();
    std::fs::create_dir_all(mid_dir.join("src")).unwrap();
    std::fs::create_dir_all(leaf_dir.join("src")).unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"registry-feature-topology-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_feature_mid = { package = \"demo-feature-mid\", version = \"=0.1.0\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_feature_mid::value() + 1 }\n").unwrap();
    std::fs::write(
        app_dir.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
    )
    .unwrap();

    std::fs::write(
        mid_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-feature-mid\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_feature_mid\"\npath = \"src/lib.rs\"\n\n[features]\ndefault = [\"dep:demo_feature_leaf\"]\n\n[dependencies]\ndemo_feature_leaf = { package = \"demo-feature-leaf\", version = \"=0.1.0\", optional = true }\n",
    )
    .unwrap();
    std::fs::write(mid_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_feature_leaf::value() + 1 }\n").unwrap();
    std::fs::write(mid_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"featuremidchecksum\"}\n")
        .unwrap();

    std::fs::write(
        leaf_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-feature-leaf\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_feature_leaf\"\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(leaf_dir.join("src/lib.rs"), "pub fn value() -> u32 { 40 }\n").unwrap();
    std::fs::write(leaf_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"featureleafchecksum\"}\n")
        .unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"demo-feature-leaf\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"featureleafchecksum\"\n\n[[package]]\nname = \"demo-feature-mid\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"featuremidchecksum\"\ndependencies = [\n \"demo-feature-leaf\",\n]\n\n[[package]]\nname = \"registry-feature-topology-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-feature-mid\",\n]\n",
    )
    .unwrap();
    app_dir
}

fn write_workspace_dependency_vendored_registry_fixture(
    dir: &TempDir,
    unsupported_inheritance: bool,
) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-workspace-dependency-topology-app");
    let member_dir = app_dir.join("member");
    let leaf_dir = app_dir.join("vendor/demo-ws-leaf-0.1.0");
    std::fs::create_dir_all(member_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join(".cargo")).unwrap();
    std::fs::create_dir_all(leaf_dir.join("src")).unwrap();
    let workspace_dependency = "demo_ws_leaf = { package = \"demo-ws-leaf\", version = \"=0.1.0\", features = [\"extra\"], default-features = false }";
    std::fs::write(
        app_dir.join("Cargo.toml"),
        format!(
            "[workspace]\nmembers = [\"member\"]\nresolver = \"2\"\n\n[workspace.dependencies]\n{workspace_dependency}\n"
        ),
    )
    .unwrap();
    std::fs::write(
        member_dir.join("Cargo.toml"),
        if unsupported_inheritance {
            "[package]\nname = \"registry-workspace-dependency-topology-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_ws_leaf = { workspace = true, features = [\"member-side\"] }\n"
        } else {
            "[package]\nname = \"registry-workspace-dependency-topology-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_ws_leaf = { workspace = true }\n"
        },
    )
    .unwrap();
    std::fs::write(member_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_ws_leaf::value() + 1 }\n").unwrap();
    std::fs::write(
        app_dir.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
    )
    .unwrap();
    std::fs::write(
        leaf_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-ws-leaf\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_ws_leaf\"\npath = \"src/lib.rs\"\n\n[features]\nextra = []\nmember-side = []\n",
    )
    .unwrap();
    std::fs::write(leaf_dir.join("src/lib.rs"), "pub fn value() -> u32 { 40 }\n").unwrap();
    std::fs::write(leaf_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"wsleafchecksum\"}\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"demo-ws-leaf\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"wsleafchecksum\"\n\n[[package]]\nname = \"registry-workspace-dependency-topology-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-ws-leaf\",\n]\n",
    )
    .unwrap();
    app_dir
}

fn write_workspace_member_glob_fixture(dir: &TempDir, unsupported_glob: bool) -> std::path::PathBuf {
    let app_dir = dir.path().join("workspace-member-glob-root");
    let app_member = app_dir.join("crates/app1");
    let dep_member = app_dir.join("crates/dep1");
    std::fs::create_dir_all(app_member.join("src")).unwrap();
    std::fs::create_dir_all(dep_member.join("src")).unwrap();
    let members = if unsupported_glob {
        ("crates/app?", "crates/dep1")
    } else {
        ("crates/*", "")
    };
    let members_line = if members.1.is_empty() {
        format!("members = [\"{}\"]", members.0)
    } else {
        format!("members = [\"{}\", \"{}\"]", members.0, members.1)
    };
    std::fs::write(app_dir.join("Cargo.toml"), format!("[workspace]\n{members_line}\nresolver = \"2\"\n")).unwrap();
    std::fs::write(
        app_member.join("Cargo.toml"),
        "[package]\nname = \"workspace-member-glob-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nworkspace_member_glob_dep = { package = \"workspace-member-glob-dep\", path = \"../dep1\" }\n",
    )
    .unwrap();
    std::fs::write(app_member.join("src/lib.rs"), "pub fn value() -> u32 { workspace_member_glob_dep::value() + 1 }\n")
        .unwrap();
    std::fs::write(
        dep_member.join("Cargo.toml"),
        "[package]\nname = \"workspace-member-glob-dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"workspace_member_glob_dep\"\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(dep_member.join("src/lib.rs"), "pub fn value() -> u32 { 40 }\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"workspace-member-glob-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"workspace-member-glob-dep\",\n]\n\n[[package]]\nname = \"workspace-member-glob-dep\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    app_dir
}

fn write_patch_source_registry_fixture(dir: &TempDir, unsupported_patch: bool) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-patch-source-topology-app");
    let patch_dir = app_dir.join("patches/demo-patch-leaf");
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::create_dir_all(patch_dir.join("src")).unwrap();
    let unsupported_patch_table = if unsupported_patch {
        "\n[patch.\"https://example.invalid/index\"]\nunused_patch = { package = \"demo-patch-leaf\", path = \"patches/demo-patch-leaf\" }\n"
    } else {
        ""
    };
    std::fs::write(
        app_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"registry-patch-source-topology-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_patch_leaf = {{ package = \"demo-patch-leaf\", version = \"=0.1.0\" }}\n\n[patch.crates-io]\ndemo_patch_leaf = {{ package = \"demo-patch-leaf\", path = \"patches/demo-patch-leaf\" }}\n{unsupported_patch_table}"
        ),
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_patch_leaf::value() + 1 }\n").unwrap();
    std::fs::write(
        patch_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-patch-leaf\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_patch_leaf\"\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(patch_dir.join("src/lib.rs"), "pub fn value() -> u32 { 40 }\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"demo-patch-leaf\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"registry-patch-source-topology-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-patch-leaf\",\n]\n",
    )
    .unwrap();
    app_dir
}

fn write_target_cfg_vendored_registry_fixture(dir: &TempDir, unsupported_cfg: bool) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-target-cfg-topology-app");
    let mid_dir = app_dir.join("vendor/demo-cfg-mid-0.1.0");
    let leaf_dir = app_dir.join("vendor/demo-cfg-leaf-0.1.0");
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join(".cargo")).unwrap();
    std::fs::create_dir_all(mid_dir.join("src")).unwrap();
    std::fs::create_dir_all(leaf_dir.join("src")).unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"registry-target-cfg-topology-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_cfg_mid = { package = \"demo-cfg-mid\", version = \"=0.1.0\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_cfg_mid::value() + 1 }\n").unwrap();
    std::fs::write(
        app_dir.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
    )
    .unwrap();

    let cfg_table = if unsupported_cfg {
        "target.'cfg(any(target_os = \"linux\", target_os = \"macos\"))'.dependencies"
    } else {
        "target.'cfg(unix)'.dependencies"
    };
    std::fs::write(
        mid_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"demo-cfg-mid\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_cfg_mid\"\npath = \"src/lib.rs\"\n\n[{cfg_table}]\ndemo_cfg_leaf = {{ package = \"demo-cfg-leaf\", version = \"=0.1.0\" }}\n"
        ),
    )
    .unwrap();
    std::fs::write(mid_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_cfg_leaf::value() + 1 }\n").unwrap();
    std::fs::write(mid_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"cfgmidchecksum\"}\n").unwrap();

    std::fs::write(
        leaf_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-cfg-leaf\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_cfg_leaf\"\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(leaf_dir.join("src/lib.rs"), "pub fn value() -> u32 { 40 }\n").unwrap();
    std::fs::write(leaf_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"cfgleafchecksum\"}\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"demo-cfg-leaf\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"cfgleafchecksum\"\n\n[[package]]\nname = \"demo-cfg-mid\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"cfgmidchecksum\"\ndependencies = [\n \"demo-cfg-leaf\",\n]\n\n[[package]]\nname = \"registry-target-cfg-topology-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-cfg-mid\",\n]\n",
    )
    .unwrap();
    app_dir
}

fn write_transitive_vendored_registry_fixture(dir: &TempDir, unsupported_leaf_bench: bool) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-transitive-topology-app");
    let mid_dir = app_dir.join("vendor/demo-mid-0.1.0");
    let leaf_dir = app_dir.join("vendor/demo-leaf-0.1.0");
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join(".cargo")).unwrap();
    std::fs::create_dir_all(mid_dir.join("src")).unwrap();
    std::fs::create_dir_all(leaf_dir.join("src")).unwrap();
    if unsupported_leaf_bench {
        std::fs::create_dir_all(leaf_dir.join("benches")).unwrap();
    }
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"registry-transitive-topology-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_mid = { package = \"demo-mid\", version = \"=0.1.0\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_mid::value() + 1 }\n").unwrap();
    std::fs::write(
        app_dir.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
    )
    .unwrap();

    std::fs::write(
        mid_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-mid\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_mid\"\npath = \"src/lib.rs\"\n\n[dependencies]\ndemo_leaf = { package = \"demo-leaf\", version = \"=0.1.0\" }\n",
    )
    .unwrap();
    std::fs::write(mid_dir.join("src/lib.rs"), "pub fn value() -> u32 { demo_leaf::value() + 1 }\n").unwrap();
    std::fs::write(mid_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"midchecksum\"}\n").unwrap();

    let bench = if unsupported_leaf_bench {
        "\n[[bench]]\nname = \"unsupported\"\npath = \"benches/unsupported.rs\"\n"
    } else {
        ""
    };
    std::fs::write(
        leaf_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"demo-leaf\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_leaf\"\npath = \"src/lib.rs\"\n{bench}"
        ),
    )
    .unwrap();
    std::fs::write(leaf_dir.join("src/lib.rs"), "pub fn value() -> u32 { 40 }\n").unwrap();
    if unsupported_leaf_bench {
        std::fs::write(leaf_dir.join("benches/unsupported.rs"), "fn main() {}\n").unwrap();
    }
    std::fs::write(leaf_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"leafchecksum\"}\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"demo-leaf\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"leafchecksum\"\n\n[[package]]\nname = \"demo-mid\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"midchecksum\"\ndependencies = [\n \"demo-leaf\",\n]\n\n[[package]]\nname = \"registry-transitive-topology-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-mid\",\n]\n",
    )
    .unwrap();
    app_dir
}

fn write_vendored_registry_proc_macro_fixture(dir: &TempDir, unsupported_registry_bench: bool) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-proc-macro-app");
    let vendor_dir = app_dir.join("vendor/demo-macro-0.1.0");
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join(".cargo")).unwrap();
    std::fs::create_dir_all(vendor_dir.join("src")).unwrap();
    if unsupported_registry_bench {
        std::fs::create_dir_all(vendor_dir.join("benches")).unwrap();
    }
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"registry-proc-macro-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_macro = { package = \"demo-macro\", version = \"=0.1.0\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "use demo_macro::demo;\n#[demo]\npub fn value() -> u32 { 3 }\n")
        .unwrap();
    std::fs::write(
        app_dir.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
    )
    .unwrap();
    let bench = if unsupported_registry_bench {
        "\n[[bench]]\nname = \"unsupported\"\npath = \"benches/unsupported.rs\"\n"
    } else {
        ""
    };
    std::fs::write(
        vendor_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"demo-macro\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_macro\"\npath = \"src/lib.rs\"\nproc-macro = true\n{bench}"
        ),
    )
    .unwrap();
    std::fs::write(
        vendor_dir.join("src/lib.rs"),
        "extern crate proc_macro;\nuse proc_macro::TokenStream;\n#[proc_macro_attribute]\npub fn demo(_attr: TokenStream, item: TokenStream) -> TokenStream { item }\n",
    )
    .unwrap();
    if unsupported_registry_bench {
        std::fs::write(vendor_dir.join("benches/unsupported.rs"), "fn main() {}\n").unwrap();
    }
    std::fs::write(vendor_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"fakechecksum\"}\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"registry-proc-macro-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-macro\",\n]\n\n[[package]]\nname = \"demo-macro\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"fakechecksum\"\n",
    )
    .unwrap();
    app_dir
}

fn write_vendored_registry_build_script_fixture(dir: &TempDir, unsupported_registry_bench: bool) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-host-artifact-app");
    let vendor_dir = app_dir.join("vendor/demo-build-0.1.0");
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join(".cargo")).unwrap();
    std::fs::create_dir_all(vendor_dir.join("src")).unwrap();
    if unsupported_registry_bench {
        std::fs::create_dir_all(vendor_dir.join("benches")).unwrap();
    }
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"registry-host-artifact-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_build = { package = \"demo-build\", version = \"=0.1.0\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn value() -> &'static str { demo_build::value() }\n").unwrap();
    std::fs::write(
        app_dir.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
    )
    .unwrap();
    let bench = if unsupported_registry_bench {
        "\n[[bench]]\nname = \"unsupported\"\npath = \"benches/unsupported.rs\"\n"
    } else {
        ""
    };
    std::fs::write(
        vendor_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"demo-build\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n\n[lib]\nname = \"demo_build\"\npath = \"src/lib.rs\"\n{bench}"
        ),
    )
    .unwrap();
    std::fs::write(
        vendor_dir.join("build.rs"),
        "fn main() {\n    let out = std::env::var(\"OUT_DIR\").unwrap();\n    std::fs::write(format!(\"{out}/generated.txt\"), \"registry-generated-ok\").unwrap();\n    println!(\"cargo:rustc-cfg=registry_build_script\");\n    println!(\"cargo:rustc-env=REGISTRY_BUILD_VALUE=registry-env-ok\");\n    println!(\"cargo:rerun-if-changed=build.rs\");\n}\n",
    )
    .unwrap();
    std::fs::write(
        vendor_dir.join("src/lib.rs"),
        "#[cfg(not(registry_build_script))]\ncompile_error!(\"missing registry build script cfg\");\npub fn value() -> &'static str {\n    let generated = include_str!(concat!(env!(\"OUT_DIR\"), \"/generated.txt\"));\n    if generated == \"registry-generated-ok\" { env!(\"REGISTRY_BUILD_VALUE\") } else { \"bad\" }\n}\n",
    )
    .unwrap();
    if unsupported_registry_bench {
        std::fs::write(vendor_dir.join("benches/unsupported.rs"), "fn main() {}\n").unwrap();
    }
    std::fs::write(vendor_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"fakechecksum\"}\n").unwrap();
    std::fs::write(
        app_dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"registry-host-artifact-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-build\",\n]\n\n[[package]]\nname = \"demo-build\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"fakechecksum\"\n",
    )
    .unwrap();
    app_dir
}

fn write_build_dependency_vendored_registry_fixture(
    dir: &TempDir,
    unsupported_build_dependency: bool,
) -> std::path::PathBuf {
    let app_dir = dir.path().join("registry-build-dependency-topology-app");
    let build_dir = app_dir.join("vendor/demo-build-with-dep-0.1.0");
    let helper_dir = app_dir.join("vendor/demo-build-helper-0.1.0");
    std::fs::create_dir_all(app_dir.join("src")).unwrap();
    std::fs::create_dir_all(app_dir.join(".cargo")).unwrap();
    std::fs::create_dir_all(build_dir.join("src")).unwrap();
    std::fs::create_dir_all(helper_dir.join("src")).unwrap();
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"registry-build-dependency-topology-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndemo_build_with_dep = { package = \"demo-build-with-dep\", version = \"=0.1.0\" }\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("src/lib.rs"), "pub fn value() -> &'static str { demo_build_with_dep::value() }\n")
        .unwrap();
    std::fs::write(
        app_dir.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
    )
    .unwrap();
    let build_dependencies = if unsupported_build_dependency {
        "[build-dependencies]\ndemo_build_helper = { package = \"demo-build-helper\", version = \"=0.1.0\", features = [\"unsupported\"] }\n"
    } else {
        "[build-dependencies]\ndemo_build_helper = { package = \"demo-build-helper\", version = \"=0.1.0\" }\n"
    };
    std::fs::write(
        build_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"demo-build-with-dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n\n[lib]\nname = \"demo_build_with_dep\"\npath = \"src/lib.rs\"\n\n{build_dependencies}"
        ),
    )
    .unwrap();
    std::fs::write(
        build_dir.join("build.rs"),
        "fn main() {\n    let out = std::env::var(\"OUT_DIR\").unwrap();\n    std::fs::write(format!(\"{out}/generated.txt\"), demo_build_helper::message()).unwrap();\n    println!(\"cargo:rustc-cfg=build_dependency_ready\");\n}\n",
    )
    .unwrap();
    std::fs::write(
        build_dir.join("src/lib.rs"),
        "#[cfg(not(build_dependency_ready))]\ncompile_error!(\"missing build dependency metadata\");\npub fn value() -> &'static str { include_str!(concat!(env!(\"OUT_DIR\"), \"/generated.txt\")) }\n",
    )
    .unwrap();
    std::fs::write(build_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"builddepchecksum\"}\n")
        .unwrap();
    std::fs::write(
        helper_dir.join("Cargo.toml"),
        "[package]\nname = \"demo-build-helper\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"demo_build_helper\"\npath = \"src/lib.rs\"\n\n[features]\nunsupported = []\n",
    )
    .unwrap();
    std::fs::write(helper_dir.join("src/lib.rs"), "pub fn message() -> &'static str { \"build-dependency-ok\" }\n")
        .unwrap();
    std::fs::write(helper_dir.join(".cargo-checksum.json"), "{\"files\":{},\"package\":\"buildhelperchecksum\"}\n")
        .unwrap();
    let missing_lock = " \"demo-build-helper\",\n";
    let helper_package = "[[package]]\nname = \"demo-build-helper\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"buildhelperchecksum\"\n";
    std::fs::write(
        app_dir.join("Cargo.lock"),
        format!(
            "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"registry-build-dependency-topology-app\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo-build-with-dep\",\n]\n\n[[package]]\nname = \"demo-build-with-dep\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"builddepchecksum\"\ndependencies = [\n{missing_lock}]\n\n{helper_package}"
        ),
    )
    .unwrap();
    app_dir
}

#[test]
fn rust_plan_cli_executes_build_dependency_vendored_registry_dependency_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_build_dependency_vendored_registry_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-build-dependency-topology-output"))
        .output()
        .expect("rust-plan build-dependency registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let registry_sources = &receipt["rust_plan"]["native_registry_source_planning"];
    assert!(registry_sources["ready"].as_bool().unwrap(), "{receipt:#?}");
    let sources = registry_sources["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2, "{receipt:#?}");
    assert!(sources.iter().any(|source| source["checksum"] == "builddepchecksum"));
    assert!(sources.iter().any(|source| source["checksum"] == "buildhelperchecksum"));

    let packages = receipt["rust_plan"]["native_package_target_planning"]["packages"].as_array().unwrap();
    let build_package = packages
        .iter()
        .find(|package| package["name"] == "demo-build-with-dep")
        .expect("build-script registry package should have native facts");
    assert!(build_package["build_dependencies"].as_array().unwrap().iter().any(|dependency| {
        dependency["name"] == "demo_build_helper"
            && dependency["manifest_path"].as_str().unwrap().contains("demo-build-helper-0.1.0")
    }));
    let host_units = receipt["rust_plan"]["native_host_unit_graph_planning"]["host_units"].as_array().unwrap();
    let build_host = host_units
        .iter()
        .find(|unit| unit["package_id"].as_str().unwrap().contains("demo-build-with-dep"))
        .expect("build script host unit should be present");
    assert!(build_host["dependency_artifacts"].as_array().unwrap().iter().any(|artifact| {
        artifact["package_id"].as_str().unwrap().contains("demo-build-helper")
            && artifact["name"] == "demo_build_helper"
    }));

    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 4, "{receipt:#?}");
    assert!(unit_executions[0]["package_id"].as_str().unwrap().contains("demo-build-helper"), "{receipt:#?}");
    assert_eq!(unit_executions[1]["target_kind"], "custom-build", "{receipt:#?}");
    assert!(unit_executions[1]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_build_helper")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
    assert!(
        topology["build_script_metadata_runs"].as_array().unwrap()[0]["out_dir_artifact_digests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|artifact| artifact["blake3"].as_str().unwrap().len() == 64)
    );
}

#[test]
fn rust_plan_cli_blocks_unsupported_build_dependency_vendored_registry_topology_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_build_dependency_vendored_registry_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-build-dependency-blocked-output"))
        .output()
        .expect("rust-plan build-dependency registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers.iter().any(|blocker| blocker["class"] == "unsupported-build-dependency-options"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_vendored_registry_build_script_host_artifact_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_vendored_registry_build_script_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-host-artifact-output"))
        .output()
        .expect("rust-plan registry host-artifact topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    assert!(receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    let registry_sources = receipt["rust_plan"]["native_registry_source_planning"]["sources"].as_array().unwrap();
    assert_eq!(registry_sources.len(), 1, "{receipt:#?}");
    assert_eq!(registry_sources[0]["checksum"], "fakechecksum");
    assert_eq!(registry_sources[0]["source_digest"]["algorithm"], "blake3-tree-v1");
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let metadata_runs = topology["build_script_metadata_runs"].as_array().unwrap();
    assert_eq!(metadata_runs.len(), 1, "{receipt:#?}");
    assert_eq!(metadata_runs[0]["rustc_cfg"].as_array().unwrap()[0], "registry_build_script");
    assert_eq!(metadata_runs[0]["rustc_env"]["REGISTRY_BUILD_VALUE"], "registry-env-ok");
    assert_eq!(metadata_runs[0]["out_dir_artifact_digests"].as_array().unwrap().len(), 1);
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2, "{receipt:#?}");
    let registry_units = unit_executions
        .iter()
        .filter(|unit| unit["package_id"].as_str().unwrap().starts_with("registry+"))
        .collect::<Vec<_>>();
    assert_eq!(registry_units.len(), 2, "{receipt:#?}");
    assert!(registry_units.iter().all(|unit| unit["source_digest"]["algorithm"] == "blake3-tree-v1"));
    assert!(registry_units.iter().any(|unit| {
        unit["host_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
            artifact["path"].as_str().unwrap().contains("build_script_build")
                && artifact["blake3"].as_str().unwrap().len() == 64
        })
    }));
}

#[test]
fn rust_plan_cli_executes_vendored_registry_build_script_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_vendored_registry_build_script_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-unified-host-output"))
        .output()
        .expect("rust-plan registry unified host topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    assert!(receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    let registry_sources = receipt["rust_plan"]["native_registry_source_planning"]["sources"].as_array().unwrap();
    assert_eq!(registry_sources.len(), 1, "{receipt:#?}");
    assert_eq!(registry_sources[0]["checksum"], "fakechecksum");
    assert_eq!(registry_sources[0]["source_digest"]["algorithm"], "blake3-tree-v1");
    assert!(registry_sources[0]["vendor_root"].as_str().unwrap().ends_with("/vendor"));

    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let metadata_runs = topology["build_script_metadata_runs"].as_array().unwrap();
    assert_eq!(metadata_runs.len(), 1, "{receipt:#?}");
    assert_eq!(metadata_runs[0]["rustc_cfg"].as_array().unwrap()[0], "registry_build_script");
    assert_eq!(metadata_runs[0]["rustc_env"]["REGISTRY_BUILD_VALUE"], "registry-env-ok");
    assert_eq!(metadata_runs[0]["out_dir_artifact_digests"].as_array().unwrap().len(), 1);

    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 3, "{receipt:#?}");
    assert_eq!(
        unit_executions[0]["target_kind"], "custom-build",
        "host producer should execute before target consumers: {receipt:#?}"
    );
    let registry_units = unit_executions
        .iter()
        .filter(|unit| unit["package_id"].as_str().unwrap().starts_with("registry+"))
        .collect::<Vec<_>>();
    assert_eq!(registry_units.len(), 2, "{receipt:#?}");
    assert!(registry_units.iter().all(|unit| unit["source_digest"]["algorithm"] == "blake3-tree-v1"));
    assert!(registry_units.iter().any(|unit| {
        unit["host_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
            artifact["path"].as_str().unwrap().contains("build_script_build")
                && artifact["blake3"].as_str().unwrap().len() == 64
        })
    }));
    assert!(unit_executions.iter().any(|unit| {
        unit["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
            artifact["path"].as_str().unwrap().contains("libdemo_build")
                && artifact["blake3"].as_str().unwrap().len() == 64
        })
    }));
}

#[test]
fn rust_plan_cli_executes_vendored_registry_proc_macro_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_vendored_registry_proc_macro_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-unified-proc-macro-output"))
        .output()
        .expect("rust-plan registry proc-macro unified topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    assert!(receipt["rust_plan"]["native_host_unit_graph_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    assert!(receipt["rust_plan"]["unit_derivation_graph"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    let registry_sources = receipt["rust_plan"]["native_registry_source_planning"]["sources"].as_array().unwrap();
    assert_eq!(registry_sources.len(), 1, "{receipt:#?}");
    assert_eq!(registry_sources[0]["checksum"], "fakechecksum");
    assert_eq!(registry_sources[0]["source_digest"]["algorithm"], "blake3-tree-v1");
    assert!(registry_sources[0]["vendor_root"].as_str().unwrap().ends_with("/vendor"));

    let host_units = receipt["rust_plan"]["native_host_unit_graph_planning"]["host_units"].as_array().unwrap();
    assert_eq!(host_units.len(), 1, "{receipt:#?}");
    assert_eq!(host_units[0]["target_kind"], "proc-macro");
    assert!(host_units[0]["package_id"].as_str().unwrap().starts_with("registry+"));
    assert_eq!(host_units[0]["source_digest"]["algorithm"], "blake3-tree-v1");

    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2, "{receipt:#?}");
    assert_eq!(
        unit_executions[0]["target_kind"], "proc-macro",
        "registry proc-macro host producer should execute before target consumer: {receipt:#?}"
    );
    assert_eq!(unit_executions[1]["target_kind"], "lib");
    assert!(unit_executions[0]["package_id"].as_str().unwrap().starts_with("registry+"));
    assert_eq!(unit_executions[0]["source_digest"]["algorithm"], "blake3-tree-v1");
    assert!(unit_executions[0]["output_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_macro") && artifact["blake3"].as_str().unwrap().len() == 64
    }));
    assert!(unit_executions[1]["host_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_macro") && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_blocks_unsupported_vendored_registry_proc_macro_layout_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_vendored_registry_proc_macro_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-unified-proc-macro-blocked-output"))
        .output()
        .expect("rust-plan registry proc-macro unified topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers.iter().any(|blocker| blocker["class"] == "unsupported-cargo-oracle-target-kind"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_blocks_unsupported_vendored_registry_host_artifact_layout_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_vendored_registry_build_script_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-host-artifact-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-host-artifact-blocked-output"))
        .output()
        .expect("rust-plan registry host-artifact topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers.iter().any(|blocker| blocker["class"] == "unsupported-cargo-oracle-target-kind"),
        "{receipt:#?}"
    );
    let topology = &receipt["host_artifact_topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_blocks_unsupported_vendored_registry_unified_host_layout_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_vendored_registry_build_script_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-unified-host-blocked-output"))
        .output()
        .expect("rust-plan registry unified host topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers.iter().any(|blocker| blocker["class"] == "unsupported-cargo-oracle-target-kind"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_vendored_registry_dependency_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_vendored_registry_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-topology-output"))
        .output()
        .expect("rust-plan registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let registry_sources = &receipt["rust_plan"]["native_registry_source_planning"];
    assert!(registry_sources["ready"].as_bool().unwrap(), "{receipt:#?}");
    assert_eq!(registry_sources["sources"].as_array().unwrap().len(), 1);
    assert_eq!(registry_sources["sources"][0]["checksum"], "fakechecksum");
    assert_eq!(registry_sources["sources"][0]["source_digest"]["algorithm"], "blake3-tree-v1");
    assert!(registry_sources["sources"][0]["vendor_root"].as_str().unwrap().ends_with("/vendor"));
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2, "{receipt:#?}");
    let registry_unit = unit_executions
        .iter()
        .find(|unit| unit["package_id"].as_str().unwrap().starts_with("registry+"))
        .expect("registry producer unit should execute");
    assert_eq!(registry_unit["execution_status"], "success");
    assert_eq!(registry_unit["source_digest"]["algorithm"], "blake3-tree-v1");
    assert_eq!(registry_unit["output_artifact_digests"].as_array().unwrap().len(), 1);
    assert!(unit_executions[1]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_dep") && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_executes_feature_gated_vendored_registry_dependency_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_feature_gated_vendored_registry_fixture(&dir);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-feature-topology-output"))
        .output()
        .expect("rust-plan feature-gated registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let registry_sources = &receipt["rust_plan"]["native_registry_source_planning"];
    assert!(registry_sources["ready"].as_bool().unwrap(), "{receipt:#?}");
    let sources = registry_sources["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2, "{receipt:#?}");
    assert!(sources.iter().any(|source| source["checksum"] == "featureleafchecksum"));
    assert!(sources.iter().any(|source| source["checksum"] == "featuremidchecksum"));

    let packages = receipt["rust_plan"]["native_package_target_planning"]["packages"].as_array().unwrap();
    let feature_mid = packages
        .iter()
        .find(|package| package["name"] == "demo-feature-mid")
        .expect("feature-gated registry package should have native facts");
    assert_eq!(feature_mid["selected_features"].as_array().unwrap()[0], "default");
    assert!(feature_mid["path_dependencies"].as_array().unwrap().iter().any(|dependency| {
        dependency["name"] == "demo_feature_leaf"
            && dependency["manifest_path"].as_str().unwrap().contains("demo-feature-leaf-0.1.0")
    }));

    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 3, "{receipt:#?}");
    assert!(unit_executions[0]["package_id"].as_str().unwrap().contains("demo-feature-leaf"), "{receipt:#?}");
    assert!(unit_executions[1]["package_id"].as_str().unwrap().contains("demo-feature-mid"), "{receipt:#?}");
    assert!(
        unit_executions[2]["package_id"].as_str().unwrap().contains("registry-feature-topology-app"),
        "{receipt:#?}"
    );
    assert!(unit_executions[1]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_feature_leaf")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
    assert!(unit_executions[2]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_feature_mid")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_blocks_all_features_vendored_registry_topology_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_feature_gated_vendored_registry_fixture(&dir);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--all-features")
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-feature-all-features-blocked-output"))
        .output()
        .expect("rust-plan feature-gated registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap(), "{receipt:#?}");
    let unit_blockers = receipt["rust_plan"]["native_unit_graph_planning"]["blockers"].as_array().unwrap();
    assert!(
        unit_blockers.iter().any(|blocker| blocker["class"] == "unsupported-feature-surface"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_workspace_dependency_vendored_registry_dependency_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_workspace_dependency_vendored_registry_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-workspace-dependency-topology-output"))
        .output()
        .expect("rust-plan workspace-dependency registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let registry_sources = &receipt["rust_plan"]["native_registry_source_planning"];
    assert!(registry_sources["ready"].as_bool().unwrap(), "{receipt:#?}");
    let sources = registry_sources["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 1, "{receipt:#?}");
    assert!(sources.iter().any(|source| source["checksum"] == "wsleafchecksum"));

    let packages = receipt["rust_plan"]["native_package_target_planning"]["packages"].as_array().unwrap();
    let member = packages
        .iter()
        .find(|package| package["name"] == "registry-workspace-dependency-topology-app")
        .expect("workspace member package should have native facts");
    assert!(member["workspace_dependencies"].as_array().unwrap().iter().any(|dependency| {
        dependency["dependency_key"] == "demo_ws_leaf"
            && dependency["inherited_package_name"] == "demo-ws-leaf"
            && dependency["inherited_features"].as_array().unwrap().iter().any(|feature| feature == "extra")
            && dependency["inherited_default_features"] == false
            && dependency["decision"] == "selected"
            && dependency["manifest_path"].as_str().unwrap().contains("demo-ws-leaf-0.1.0")
    }));
    assert!(member["path_dependencies"].as_array().unwrap().iter().any(|dependency| {
        dependency["name"] == "demo_ws_leaf"
            && dependency["manifest_path"].as_str().unwrap().contains("demo-ws-leaf-0.1.0")
    }));

    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2, "{receipt:#?}");
    assert!(unit_executions[0]["package_id"].as_str().unwrap().contains("demo-ws-leaf"), "{receipt:#?}");
    assert!(
        unit_executions[1]["package_id"]
            .as_str()
            .unwrap()
            .contains("registry-workspace-dependency-topology-app"),
        "{receipt:#?}"
    );
    assert!(unit_executions[1]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_ws_leaf")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_blocks_unsupported_workspace_dependency_vendored_registry_topology_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_workspace_dependency_vendored_registry_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-workspace-dependency-blocked-output"))
        .output()
        .expect("rust-plan workspace-dependency registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers
            .iter()
            .any(|blocker| blocker["class"] == "unsupported-workspace-dependency-options"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_workspace_member_glob_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_workspace_member_glob_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("workspace-member-glob-output"))
        .output()
        .expect("rust-plan workspace member glob topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let packages = receipt["rust_plan"]["native_package_target_planning"]["packages"].as_array().unwrap();
    assert!(packages.iter().any(|package| package["name"] == "workspace-member-glob-app"), "{receipt:#?}");
    assert!(packages.iter().any(|package| package["name"] == "workspace-member-glob-dep"), "{receipt:#?}");
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert!(
        unit_executions
            .iter()
            .any(|unit| unit["package_id"].as_str().unwrap().contains("workspace-member-glob-dep")),
        "{receipt:#?}"
    );
    assert!(
        unit_executions
            .iter()
            .any(|unit| unit["package_id"].as_str().unwrap().contains("workspace-member-glob-app")),
        "{receipt:#?}"
    );
}

#[test]
fn rust_plan_cli_blocks_unsupported_workspace_member_glob_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_workspace_member_glob_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("workspace-member-glob-blocked-output"))
        .output()
        .expect("rust-plan workspace member glob topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers.iter().any(|blocker| blocker["class"] == "unsupported-workspace-member-pattern"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_patch_source_registry_dependency_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_patch_source_registry_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-patch-source-topology-output"))
        .output()
        .expect("rust-plan patch source topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let registry_sources = &receipt["rust_plan"]["native_registry_source_planning"];
    assert!(registry_sources["ready"].as_bool().unwrap(), "{receipt:#?}");
    let sources = registry_sources["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 1, "{receipt:#?}");
    assert!(sources.iter().any(|source| {
        source["name"] == "demo-patch-leaf"
            && source["source_class"] == "patch-path"
            && source["checksum"].as_str().unwrap().starts_with("patch-path:")
            && source["manifest_path"].as_str().unwrap().contains("patches/demo-patch-leaf")
    }));

    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 2, "{receipt:#?}");
    assert!(unit_executions[0]["package_id"].as_str().unwrap().contains("demo-patch-leaf"), "{receipt:#?}");
    assert!(unit_executions[1]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_patch_leaf")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_blocks_unsupported_patch_source_registry_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_patch_source_registry_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-patch-source-blocked-output"))
        .output()
        .expect("rust-plan patch source topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let source_blockers = receipt["rust_plan"]["native_registry_source_planning"]["blockers"].as_array().unwrap();
    assert!(
        source_blockers.iter().any(|blocker| blocker["class"] == "unsupported-patch-source-registry"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_target_cfg_vendored_registry_dependency_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_target_cfg_vendored_registry_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-target-cfg-topology-output"))
        .output()
        .expect("rust-plan target-cfg registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let registry_sources = &receipt["rust_plan"]["native_registry_source_planning"];
    assert!(registry_sources["ready"].as_bool().unwrap(), "{receipt:#?}");
    let sources = registry_sources["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2, "{receipt:#?}");
    assert!(sources.iter().any(|source| source["checksum"] == "cfgleafchecksum"));
    assert!(sources.iter().any(|source| source["checksum"] == "cfgmidchecksum"));

    let packages = receipt["rust_plan"]["native_package_target_planning"]["packages"].as_array().unwrap();
    let cfg_mid = packages
        .iter()
        .find(|package| package["name"] == "demo-cfg-mid")
        .expect("target-cfg registry package should have native facts");
    assert!(cfg_mid["target_cfg_dependencies"].as_array().unwrap().iter().any(|dependency| {
        dependency["cfg"] == "cfg(unix)"
            && dependency["decision"] == "selected"
            && dependency["name"] == "demo_cfg_leaf"
            && dependency["manifest_path"].as_str().unwrap().contains("demo-cfg-leaf-0.1.0")
    }));
    assert!(cfg_mid["path_dependencies"].as_array().unwrap().iter().any(|dependency| {
        dependency["name"] == "demo_cfg_leaf"
            && dependency["manifest_path"].as_str().unwrap().contains("demo-cfg-leaf-0.1.0")
    }));

    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 3, "{receipt:#?}");
    assert!(unit_executions[0]["package_id"].as_str().unwrap().contains("demo-cfg-leaf"), "{receipt:#?}");
    assert!(unit_executions[1]["package_id"].as_str().unwrap().contains("demo-cfg-mid"), "{receipt:#?}");
    assert!(
        unit_executions[2]["package_id"].as_str().unwrap().contains("registry-target-cfg-topology-app"),
        "{receipt:#?}"
    );
    assert!(unit_executions[1]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_cfg_leaf")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
    assert!(unit_executions[2]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_cfg_mid")
            && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_blocks_unsupported_target_cfg_vendored_registry_topology_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_target_cfg_vendored_registry_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-target-cfg-blocked-output"))
        .output()
        .expect("rust-plan target-cfg registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers.iter().any(|blocker| blocker["class"] == "unsupported-target-cfg-surface"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_executes_transitive_vendored_registry_dependencies_in_unified_topology() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_transitive_vendored_registry_fixture(&dir, false);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-transitive-topology-output"))
        .output()
        .expect("rust-plan transitive registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    let registry_sources = &receipt["rust_plan"]["native_registry_source_planning"];
    assert!(registry_sources["ready"].as_bool().unwrap(), "{receipt:#?}");
    let sources = registry_sources["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2, "{receipt:#?}");
    assert!(sources.iter().any(|source| source["checksum"] == "leafchecksum"));
    assert!(sources.iter().any(|source| source["checksum"] == "midchecksum"));
    assert!(sources.iter().all(|source| source["source_digest"]["algorithm"] == "blake3-tree-v1"));
    assert!(sources.iter().all(|source| source["vendor_root"].as_str().unwrap().ends_with("/vendor")));

    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{receipt:#?}");
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
    let unit_executions = topology["unit_executions"].as_array().unwrap();
    assert_eq!(unit_executions.len(), 3, "{receipt:#?}");
    assert!(unit_executions.iter().all(|execution| execution["execution_status"] == "success"));
    assert!(unit_executions[0]["package_id"].as_str().unwrap().contains("demo-leaf"), "{receipt:#?}");
    assert!(unit_executions[1]["package_id"].as_str().unwrap().contains("demo-mid"), "{receipt:#?}");
    assert!(
        unit_executions[2]["package_id"].as_str().unwrap().contains("registry-transitive-topology-app"),
        "{receipt:#?}"
    );
    assert!(unit_executions[1]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_leaf") && artifact["blake3"].as_str().unwrap().len() == 64
    }));
    assert!(unit_executions[2]["dependency_artifact_digests"].as_array().unwrap().iter().any(|artifact| {
        artifact["path"].as_str().unwrap().contains("libdemo_mid") && artifact["blake3"].as_str().unwrap().len() == 64
    }));
}

#[test]
fn rust_plan_cli_blocks_unsupported_transitive_vendored_registry_layout_before_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_transitive_vendored_registry_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-transitive-unsupported-output"))
        .output()
        .expect("rust-plan transitive registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap());
    let registry_sources = receipt["rust_plan"]["native_registry_source_planning"]["sources"].as_array().unwrap();
    assert_eq!(registry_sources.len(), 2, "{receipt:#?}");
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers.iter().any(|blocker| blocker["class"] == "unsupported-cargo-oracle-target-kind"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
    assert_eq!(topology["build_script_metadata_runs"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_blocks_unsupported_vendored_registry_layout_before_topology_rustc() {
    let dir = TempDir::new().unwrap();
    let app_dir = write_vendored_registry_fixture(&dir, true);

    let output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&app_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(dir.path().join("registry-unsupported-output"))
        .output()
        .expect("rust-plan registry topology CLI should run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("CLI should emit JSON receipt");
    assert!(receipt["rust_plan"]["native_registry_source_planning"]["ready"].as_bool().unwrap());
    let package_blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(
        package_blockers.iter().any(|blocker| blocker["class"] == "unsupported-cargo-oracle-target-kind"),
        "{receipt:#?}"
    );
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["unit_executions"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_plan_cli_reuses_unified_topology_outputs_on_repeat_run() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("reuse-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"reuse-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("Cargo.lock"), "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"reuse-crate\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 7 }\n").unwrap();
    let output_root = dir.path().join("reuse-output");

    let first_output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(&output_root)
        .output()
        .expect("first rust-plan topology CLI should run");
    assert!(first_output.status.success(), "{}", String::from_utf8_lossy(&first_output.stderr));
    let first_receipt: Value = serde_json::from_slice(&first_output.stdout).expect("CLI should emit JSON receipt");
    let first_unit = &first_receipt["topology_execution"]["unit_executions"].as_array().unwrap()[0];
    assert_eq!(first_unit["rebuild_reason"], "rebuilt-explicit-unit");

    let second_output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(&output_root)
        .output()
        .expect("second rust-plan topology CLI should run");
    assert!(second_output.status.success(), "{}", String::from_utf8_lossy(&second_output.stderr));
    let second_receipt: Value = serde_json::from_slice(&second_output.stdout).expect("CLI should emit JSON receipt");
    let topology = &second_receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "success", "{second_receipt:#?}");
    let second_unit = &topology["unit_executions"].as_array().unwrap()[0];
    assert_eq!(second_unit["rebuild_reason"], "reused-explicit-unit-output");
    assert_eq!(
        first_unit["output_artifact_digests"], second_unit["output_artifact_digests"],
        "reuse must bind the same declared output BLAKE3 digests"
    );
}

#[test]
fn rust_plan_cli_blocks_stale_unified_topology_cached_output() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("stale-reuse-crate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"stale-reuse-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("Cargo.lock"), "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"stale-reuse-crate\"\nversion = \"0.1.0\"\n").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u32 { 9 }\n").unwrap();
    let output_root = dir.path().join("stale-output");

    let first_output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(&output_root)
        .output()
        .expect("first rust-plan topology CLI should run");
    assert!(first_output.status.success(), "{}", String::from_utf8_lossy(&first_output.stderr));
    let cached_rlib = find_first_rlib(&output_root).expect("topology run should produce an rlib");
    std::fs::remove_file(cached_rlib).unwrap();

    let second_output = mantle_cmd()
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&crate_dir)
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(&output_root)
        .output()
        .expect("second rust-plan topology CLI should run");
    assert!(second_output.status.success(), "{}", String::from_utf8_lossy(&second_output.stderr));
    let receipt: Value = serde_json::from_slice(&second_output.stdout).expect("CLI should emit JSON receipt");
    let topology = &receipt["topology_execution"];
    assert_eq!(topology["execution_status"], "blocked", "{receipt:#?}");
    assert_eq!(topology["blocker"]["class"], "stale-cached-output");
    assert_eq!(topology["unit_executions"].as_array().unwrap()[0]["rebuild_reason"], "not-run-stale-cached-output");
}

fn find_first_rlib(root: &std::path::Path) -> Option<std::path::PathBuf> {
    for entry in std::fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_first_rlib(&path) {
                return Some(found);
            }
        } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rlib") {
            return Some(path);
        }
    }
    None
}
