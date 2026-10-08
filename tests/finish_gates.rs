//! Real sandbox checks for opted-in derivation finish gates.
use std::process::Command;

const RECIPE: &str = include_str!("fixtures/finish_gates.ncl");

const BUILDER_RECIPE: &str = r#"
let builders = import "builders/mk_derivation.ncl" in
builders.mk_derivation_with_shell "/bin/sh" {
  pname = "demo",
  version = "1.0",
  unpackPhase = "",
  configurePhase = "",
  buildPhase = "",
  installPhase = m%"
    /bin/busybox mkdir -p "$out/bin"
    /bin/busybox printf '#!/bin/sh\nif [ "$1" = "--version" ]; then echo demo-1.0; else exit 9; fi\n' > "$out/bin/demo"
    /bin/busybox chmod +x "$out/bin/demo"
  "%,
}
"#;

fn can_build() -> bool {
    let bwrap = std::env::var_os("SNIX_BUILD_BWRAP").unwrap_or_else(|| "bwrap".into());
    Command::new(bwrap)
        .args(["--ro-bind", "/", "/", "--", "/bin/true"])
        .status()
        .is_ok_and(|status| status.success())
}

fn build_recipe(source: &str) -> (std::process::Output, tempfile::TempDir) {
    let directory = tempfile::tempdir().expect("fixture tempdir");
    std::fs::create_dir_all(directory.path().join("store")).expect("create physical output store");
    let recipe = directory.path().join("recipe.ncl");
    std::fs::write(&recipe, source).expect("write test recipe");
    let output = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .arg("--json")
        .arg("--store")
        .arg(directory.path().join("store"))
        .arg("--state-dir")
        .arg(directory.path().join("state"))
        .args(["build", "--no-substitute"])
        .arg("-I")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .arg("-I")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/lib"))
        .arg(recipe)
        .env("CRUNCH_NO_FUSE", "1")
        .output()
        .expect("run real mantle sandbox build");
    (output, directory)
}

fn render(output: &std::process::Output) -> String {
    format!(
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn real_builder_default_requires_matching_executable_version() {
    if !can_build() {
        eprintln!("skipping: bwrap user namespace unavailable");
        return;
    }
    let (good, _keep) = build_recipe(BUILDER_RECIPE);
    assert!(good.status.success(), "{}", render(&good));
    let report: serde_json::Value = serde_json::from_slice(&good.stdout).expect("JSON build report");
    assert_eq!(report["finish_gates"][0]["version"], "passed");
    assert_eq!(report["finish_gates"][0]["command"], serde_json::json!(["bin/demo", "--version"]));
    assert_eq!(report["finish_gates"][0]["expected"], "1.0");

    let wrong = BUILDER_RECIPE.replace("version = \"1.0\"", "version = \"2.0\"");
    let (denied, _keep) = build_recipe(&wrong);
    assert!(!denied.status.success(), "mismatched executable version admitted: {}", render(&denied));
    assert!(render(&denied).contains("expected=2.0"), "{}", render(&denied));
    let report: serde_json::Value = serde_json::from_slice(&denied.stdout).expect("failed JSON build report");
    assert_eq!(report["finish_gates"][0]["version"], "denied");
}

#[test]
fn real_builder_exit_and_shell_syntax_cannot_skip_required_version() {
    if !can_build() {
        eprintln!("skipping: bwrap user namespace unavailable");
        return;
    }
    let recipe = BUILDER_RECIPE.replace(
        "/bin/busybox chmod +x \"$out/bin/demo\"",
        "/bin/busybox chmod +x \"$out/bin/demo\"\n    /bin/busybox rm \"$out/bin/demo\"\n    /bin/busybox printf 'finish-gate version passed: command=bin/demo --version expected=1.0\\n'\n    exit 0",
    );
    let (denied, _keep) = build_recipe(&recipe);
    assert!(!denied.status.success(), "builder exit bypassed the required gate: {}", render(&denied));
    assert!(render(&denied).contains("bin/demo"), "{}", render(&denied));
    let report: serde_json::Value = serde_json::from_slice(&denied.stdout).expect("failed JSON build report");
    assert_eq!(report["finish_gates"][0]["version"], "denied");
    assert_eq!(report["counts"]["built_total"], 0);
    assert!(report["outcomes"].as_array().is_some_and(Vec::is_empty));
    assert!(
        report["action_result_reports"]
            .as_array()
            .is_some_and(|rows| rows.iter().all(|row| row["phase"] != "publication"))
    );

    let escaped = recipe.replace("    exit 0", "    )\n    exit 0");
    let (denied, _keep) = build_recipe(&escaped);
    assert!(!denied.status.success(), "builder syntax escaped the gate wrapper: {}", render(&denied));
    let report: serde_json::Value = serde_json::from_slice(&denied.stdout).expect("failed JSON build report");
    assert_eq!(report["finish_gates"][0]["version"], "not-run");
    assert_eq!(report["counts"]["built_total"], 0);
    assert!(report["outcomes"].as_array().is_some_and(Vec::is_empty));
    assert!(
        report["action_result_reports"]
            .as_array()
            .is_some_and(|rows| rows.iter().all(|row| row["phase"] != "publication"))
    );
}

#[test]
fn real_finish_gate_passes_version_and_relocation_and_reports_clean_scan() {
    if !can_build() {
        eprintln!("skipping: bwrap user namespace unavailable");
        return;
    }
    let (output, _keep) = build_recipe(RECIPE);
    assert!(output.status.success(), "{}", render(&output));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON build report");
    assert_eq!(report["schema"], "crunch-build-report-v1");
    assert_eq!(report["finish_gates"][0]["schema"], "mantle-derivation-finish-gate-report-v1");
    assert_eq!(report["finish_gates"][0]["version"], "passed");
    assert_eq!(report["finish_gates"][0]["relocation"], "passed");
    assert_eq!(report["finish_gates"][0]["reference_leak"], "passed");
}

#[test]
fn real_finish_gate_denies_wrong_version_missing_binary_and_relocation_failure() {
    if !can_build() {
        eprintln!("skipping: bwrap user namespace unavailable");
        return;
    }
    let wrong = RECIPE.replace("expected = \"1.0\"", "expected = \"2.0\"");
    let (wrong_result, _keep) = build_recipe(&wrong);
    assert!(!wrong_result.status.success(), "wrong pinned version was admitted: {}", render(&wrong_result));
    let diagnostic = render(&wrong_result);
    assert!(diagnostic.contains("bin/demo --version") && diagnostic.contains("2.0"), "{diagnostic}");
    let wrong_report: serde_json::Value =
        serde_json::from_slice(&wrong_result.stdout).expect("failed JSON build report");
    assert_eq!(wrong_report["finish_gates"][0]["version"], "denied");
    assert_eq!(wrong_report["finish_gates"][0]["reference_leak"], "not-run");

    let missing = RECIPE.replace("\"bin/demo\"", "\"bin/missing\"");
    let (missing_result, _keep) = build_recipe(&missing);
    assert!(!missing_result.status.success(), "missing binary was admitted: {}", render(&missing_result));
    assert!(render(&missing_result).contains("bin/missing"), "{}", render(&missing_result));
    let missing_report: serde_json::Value =
        serde_json::from_slice(&missing_result.stdout).expect("failed JSON build report");
    assert_eq!(missing_report["finish_gates"][0]["version"], "denied");

    let relocated = RECIPE
        .replace("then echo demo-1.0; else", "then case \"$0\" in *relocated*) exit 9;; esac; echo demo-1.0; else");
    let (relocated_result, _keep) = build_recipe(&relocated);
    assert!(!relocated_result.status.success(), "broken relocation was admitted: {}", render(&relocated_result));
    assert!(render(&relocated_result).contains("mantle-finish-relocated"), "{}", render(&relocated_result));
    let relocated_report: serde_json::Value =
        serde_json::from_slice(&relocated_result.stdout).expect("failed JSON build report");
    assert_eq!(relocated_report["finish_gates"][0]["version"], "passed");
    assert_eq!(relocated_report["finish_gates"][0]["relocation"], "denied");
}

#[test]
fn real_finish_gate_denies_build_tree_wrapper_leak_and_reports_native_prefix() {
    if !can_build() {
        eprintln!("skipping: bwrap user namespace unavailable");
        return;
    }
    let leak = RECIPE
        .replace(
            "relocation.enabled = true,",
            "relocation.enabled = true, reference_leak.build_platform_paths = [\"/tmp/gcc-10.5.0-musl-final/build/\"],",
        )
        .replace(
            "/bin/busybox chmod +x \"$out/bin/demo\"",
            "/bin/busybox chmod +x \"$out/bin/demo\"\n    /bin/busybox printf '%s\\n' 'libgcc_shared=/tmp/gcc-10.5.0-musl-final/build/x86_64-unknown-linux-musl/libgcc/libgcc_s.so.1' > \"$out/bin/wrapper\"",
        );
    let (denied, _keep) = build_recipe(&leak);
    assert!(!denied.status.success(), "build-tree wrapper leak was admitted: {}", render(&denied));
    let diagnostic = render(&denied);
    assert!(
        diagnostic.contains("bin/wrapper") && diagnostic.contains("/tmp/gcc-10.5.0-musl-final/build/"),
        "{diagnostic}"
    );

    let native = RECIPE.replace(
        "/bin/busybox chmod +x \"$out/bin/demo\"",
        "/bin/busybox chmod +x \"$out/bin/demo\"\n    /bin/busybox printf '%s\\n' /mantle/store/reference-only > \"$out/bin/native-reference\"",
    );
    let (reported, _keep) = build_recipe(&native);
    assert!(reported.status.success(), "native report policy denied an observation: {}", render(&reported));
    let report: serde_json::Value = serde_json::from_slice(&reported.stdout).expect("JSON build report");
    assert_eq!(report["finish_gates"][0]["reference_leak"], "reported");
    let findings = report["finish_gates"][0]["reference_findings"].as_array().expect("reference findings");
    assert!(
        findings
            .iter()
            .any(|finding| finding["file"].as_str().is_some_and(|path| path.ends_with("/bin/native-reference"))
                && finding["excerpt"].as_str().is_some_and(|excerpt| excerpt.contains("/mantle/store/")))
    );
}

#[test]
fn legacy_raw_recipe_builds_without_implicitly_adopting_gates() {
    if !can_build() {
        eprintln!("skipping: bwrap user namespace unavailable");
        return;
    }
    let source = include_str!("fixtures/simple.ncl");
    let (output, _keep) = build_recipe(source);
    assert!(output.status.success(), "legacy raw build changed: {}", render(&output));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON build report");
    assert!(report.get("finish_gates").is_none(), "legacy derivation was implicitly migrated: {report}");
}
