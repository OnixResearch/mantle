//! Integration tests for the crunch Nickel stdlib.
//!
//! These tests evaluate Nickel expressions through crunch-eval,
//! using the stdlib from the source tree.

use std::ffi::OsString;

fn stdlib_import_path() -> Vec<OsString> {
    let lib_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib");
    assert!(lib_dir.join("lib.ncl").exists(), "stdlib not found at {lib_dir:?}");
    vec![lib_dir.into()]
}

#[test]
fn contract_catches_missing_name() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in { builder = "/bin/sh" } | crunch.Derivation"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "should fail: missing name");
}

#[test]
fn contract_catches_wrong_type() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in { name = 42, builder = "/bin/sh" } | crunch.Derivation"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "should fail: name is not a string");
}

#[test]
fn contract_rejects_extra_fields() {
    // The Derivation contract is a closed record. Extra fields
    // (pname, version, meta, passthru, etc.) belong in the builders
    // package contract, not the core Derivation contract.
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in { name = "x", builder = "/bin/sh", bogus = true } | crunch.Derivation"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "extra fields should be rejected by closed Derivation contract");
}

#[test]
fn enum_tags_deserialize_via_json() {
    #[derive(serde::Deserialize, Debug)]
    struct Drv {
        system: String,
        sandbox: String,
    }

    let drv: Drv = crunch_eval::evaluate_str_and_deserialize(
        r#"let crunch = import "lib.ncl" in { name = "t", builder = "/bin/sh" } | crunch.Derivation"#,
        &stdlib_import_path(),
    )
    .unwrap();

    assert_eq!(drv.system, "x86_64-linux");
    assert_eq!(drv.sandbox, "native");
}

#[test]
fn enum_tags_deserialize_direct_serde() {
    // This tests the direct to_serde() path (no JSON intermediate),
    // using CrunchDerivation which has NickelString-aware deserialization
    // for enum-backed fields.
    use crunch_glue::CrunchDerivation;

    let expr = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in { name = "t", builder = "/bin/sh" } | crunch.Derivation"#,
        &stdlib_import_path(),
    )
    .unwrap();

    let drv: CrunchDerivation = expr.to_serde().unwrap();
    assert_eq!(drv.system, "x86_64-linux");
    assert_eq!(drv.name, "t");
}

#[test]
fn i386_linux_system_tag_is_accepted() {
    #[derive(serde::Deserialize, Debug)]
    struct Drv {
        system: String,
    }

    let drv: Drv = crunch_eval::evaluate_str_and_deserialize(
        r#"let crunch = import "lib.ncl" in { name = "t", builder = "/bin/sh", system = 'i386-linux } | crunch.Derivation"#,
        &stdlib_import_path(),
    )
    .unwrap();

    assert_eq!(drv.system, "i386-linux");
}

#[test]
fn store_path_validator_accepts_nix_store_path() {
    let expr = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash" | crunch.StorePath"#,
        &stdlib_import_path(),
    );
    assert!(expr.is_ok(), "valid /nix/store path should pass");
}

#[test]
fn store_path_validator_accepts_custom_store_prefix() {
    let expr = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash" | crunch.StorePath"#,
        &stdlib_import_path(),
    );
    assert!(expr.is_ok(), "valid custom store-prefix path should pass");
}

#[test]
fn store_path_validator_rejects_invalid() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in "/tmp/bad" | crunch.StorePath"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "invalid store path should fail");
}

#[test]
fn recursive_record_self_reference() {
    #[derive(serde::Deserialize, Debug)]
    struct Drv {
        name: String,
        env: std::collections::HashMap<String, String>,
    }

    let expr = crunch_eval::evaluate_str(
        r#"
        let crunch = import "lib.ncl" in
        {
            name = "myapp",
            builder = "/bin/sh",
            env = { APP_NAME = name },
        } | crunch.Derivation
        "#,
        &stdlib_import_path(),
    )
    .unwrap();

    let drv: Drv = expr.to_serde().unwrap();
    assert_eq!(drv.name, "myapp");
    assert_eq!(drv.env.get("APP_NAME").unwrap(), "myapp");
}

#[test]
fn fixed_output_contract() {
    #[derive(serde::Deserialize, Debug)]
    struct Fo {
        hash: String,
        algo: String,
        mode: String,
    }
    #[derive(serde::Deserialize, Debug)]
    struct Drv {
        fixed_output: Option<Fo>,
    }

    let drv: Drv = crunch_eval::evaluate_str_and_deserialize(
        r#"
        let crunch = import "lib.ncl" in
        {
            name = "src",
            builder = "/bin/sh",
            fixed_output = { hash = "sha256-abc123", algo = 'sha256 },
        } | crunch.Derivation
        "#,
        &stdlib_import_path(),
    )
    .unwrap();

    let fo = drv.fixed_output.unwrap();
    assert_eq!(fo.hash, "sha256-abc123");
    assert_eq!(fo.algo, "sha256");
    assert_eq!(fo.mode, "flat"); // default
}

#[test]
fn full_serde_round_trip() {
    use crunch_glue::CrunchDerivation;

    let drv: CrunchDerivation = crunch_eval::evaluate_str_and_deserialize(
        r#"
        let crunch = import "lib.ncl" in
        {
            name = "hello",
            builder = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash/bin/bash",
            args = ["-c", "echo hi > $out"],
            inputs = ["/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash"],
            addressing_mode = 'input-addressed,
        } | crunch.Derivation
        "#,
        &stdlib_import_path(),
    )
    .unwrap();

    assert_eq!(drv.name, "hello");
    assert_eq!(drv.args.len(), 2);
    assert_eq!(drv.inputs.len(), 1);

    // Convert through glue
    let mut kp = crunch_glue::ConversionCache::default();
    let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut kp).unwrap();
    assert!(drv_path.to_string().ends_with("hello.drv"));
    assert!(nix_drv.outputs.get("out").unwrap().path.is_some());
}

// ── Fetcher stdlib tests ───────────────────────────────────────────────

#[test]
fn fetchurl_produces_fod_record() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in
           crunch.fetchurl {
             url = "https://example.com/foo.txt",
             hash = "sha256-Q3QXOoy+iN4VK2CflvRulYvPZXYgF0dO7FoF7CvWFTA=",
           }"#,
        &stdlib_import_path(),
    );
    assert!(result.is_ok(), "fetchurl failed: {:?}", result.err());
    let drv: crunch_glue::CrunchDerivation = result.unwrap().to_serde().unwrap();
    assert_eq!(drv.builder, "builtin:fetchurl");
    assert_eq!(drv.name, "foo.txt");
    assert!(drv.fixed_output.is_some());
    let fo = drv.fixed_output.unwrap();
    assert_eq!(fo.mode, "flat");
    assert_eq!(drv.env.get("url").unwrap(), "https://example.com/foo.txt");
}

#[test]
fn fetchurl_preserves_optional_fetch_policy_env() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in
           crunch.fetchurl {
             url = "https://example.com/foo.txt",
             hash = "sha256-Q3QXOoy+iN4VK2CflvRulYvPZXYgF0dO7FoF7CvWFTA=",
             fetch_policy = "build-fetch-action",
           }"#,
        &stdlib_import_path(),
    );
    assert!(result.is_ok(), "fetchurl failed: {:?}", result.err());
    let drv: crunch_glue::CrunchDerivation = result.unwrap().to_serde().unwrap();
    assert_eq!(drv.env.get("fetch_policy").map(String::as_str), Some("build-fetch-action"));
    assert_eq!(drv.env.get("url").map(String::as_str), Some("https://example.com/foo.txt"));
}

#[test]
fn fetchurl_custom_name() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in
           crunch.fetchurl {
             url = "https://example.com/foo.txt",
             hash = "sha256-Q3QXOoy+iN4VK2CflvRulYvPZXYgF0dO7FoF7CvWFTA=",
             name = "my-source",
           }"#,
        &stdlib_import_path(),
    );
    assert!(result.is_ok(), "fetchurl failed: {:?}", result.err());
    let drv: crunch_glue::CrunchDerivation = result.unwrap().to_serde().unwrap();
    assert_eq!(drv.name, "my-source");
}

#[test]
fn fetch_tarball_produces_recursive_hash() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in
           crunch.fetchTarball {
             url = "https://example.com/src.tar.gz",
             hash = "sha256-Q3QXOoy+iN4VK2CflvRulYvPZXYgF0dO7FoF7CvWFTA=",
           }"#,
        &stdlib_import_path(),
    );
    assert!(result.is_ok(), "fetchTarball failed: {:?}", result.err());
    let drv: crunch_glue::CrunchDerivation = result.unwrap().to_serde().unwrap();
    assert_eq!(drv.builder, "builtin:fetchurl");
    assert!(drv.fixed_output.is_some());
    let fo = drv.fixed_output.unwrap();
    assert_eq!(fo.mode, "recursive");
    assert_eq!(drv.env.get("unpack").unwrap(), "1");
    // Name derived from URL without .tar.gz suffix.
    assert_eq!(drv.name, "src");
}

#[test]
fn fetch_git_produces_git_env() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in
           crunch.fetchGit {
             url = "https://github.com/user/repo.git",
             rev = "abc123def456",
             hash = "sha256-Q3QXOoy+iN4VK2CflvRulYvPZXYgF0dO7FoF7CvWFTA=",
           }"#,
        &stdlib_import_path(),
    );
    assert!(result.is_ok(), "fetchGit failed: {:?}", result.err());
    let drv: crunch_glue::CrunchDerivation = result.unwrap().to_serde().unwrap();
    assert_eq!(drv.builder, "builtin:fetchurl");
    assert_eq!(drv.system, "builtin");
    assert_eq!(drv.env.get("url").unwrap(), "https://github.com/user/repo.git");
    assert_eq!(drv.env.get("type").unwrap(), "git");
    assert_eq!(drv.env.get("rev").unwrap(), "abc123def456");
    let fixed_output = drv.fixed_output.expect("fetchGit must remain fixed-output");
    assert_eq!(fixed_output.mode, "recursive");
    assert_eq!(fixed_output.hash, "sha256-Q3QXOoy+iN4VK2CflvRulYvPZXYgF0dO7FoF7CvWFTA=");
    assert_eq!(drv.name, "repo");
}

#[test]
fn fetchurl_rejects_empty_url() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in
           crunch.fetchurl {
             url = "",
             hash = "sha256-Q3QXOoy+iN4VK2CflvRulYvPZXYgF0dO7FoF7CvWFTA=",
           }"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "should reject empty URL");
}

#[test]
fn fetchurl_invalid_hash_passes_eval() {
    // Invalid hashes are not caught at Nickel eval time — the Rust glue
    // layer validates them. This test confirms the Nickel layer does not
    // reject bad hashes (defense in depth is on the Rust side).
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in
           crunch.fetchurl {
             url = "https://example.com/foo.txt",
             hash = "not-a-hash",
           }"#,
        &stdlib_import_path(),
    );
    assert!(result.is_ok(), "eval should succeed; Rust layer validates hash format");
}

// ── Output selection stdlib tests ─────────────────────────────────────

#[test]
fn select_produces_correct_structure() {
    #[derive(serde::Deserialize, Debug)]
    struct OutputRef {
        drv: crunch_glue::CrunchDerivation,
        output: String,
    }

    let oref: OutputRef = crunch_eval::evaluate_str_and_deserialize(
        r#"let crunch = import "lib.ncl" in
           let pkg = { name = "libfoo", builder = "/bin/sh", outputs = ["out", "dev"] } | crunch.Derivation in
           crunch.select pkg "dev""#,
        &stdlib_import_path(),
    )
    .unwrap();

    assert_eq!(oref.output, "dev");
    assert_eq!(oref.drv.name, "libfoo");
    assert_eq!(oref.drv.outputs, vec!["out", "dev"]);
}

#[test]
fn mixed_inputs_array_validates() {
    // All three Input forms pass the contract in a single array.
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in
           let dep = { name = "dep", builder = "/bin/sh", outputs = ["out", "dev"] } | crunch.Derivation in
           {
             name = "consumer",
             builder = "/bin/sh",
             inputs = [
               "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash",
               dep,
               crunch.select dep "dev",
             ],
           } | crunch.Derivation"#,
        &stdlib_import_path(),
    );
    assert!(result.is_ok(), "mixed inputs should validate: {:?}", result.err());
}

#[test]
fn select_round_trip_through_glue() {
    // Nickel select → serde → crunch-glue convert: the selected output
    // appears in input_derivations with only that output name.
    use crunch_glue::CrunchDerivation;
    use crunch_glue::Input;

    let drv: CrunchDerivation = crunch_eval::evaluate_str_and_deserialize(
        r#"let crunch = import "lib.ncl" in
           let dep = { name = "libfoo", builder = "/bin/sh", outputs = ["out", "dev", "lib"], addressing_mode = 'input-addressed } | crunch.Derivation in
           {
             name = "consumer",
             builder = "/bin/sh",
             addressing_mode = 'input-addressed,
             inputs = [ crunch.select dep "dev" ],
           } | crunch.Derivation"#,
        &stdlib_import_path(),
    ).unwrap();

    // Verify the Input deserialized as OutputSelection.
    assert_eq!(drv.inputs.len(), 1);
    assert!(
        matches!(&drv.inputs[0], Input::OutputSelection(oref) if oref.output == "dev"),
        "expected OutputSelection, got: {:?}",
        drv.inputs[0]
    );

    // Convert through glue and check input_derivations.
    let mut kp = crunch_glue::ConversionCache::default();
    let (_, nix_drv) = crunch_glue::convert(&drv, &mut kp).unwrap();
    assert_eq!(nix_drv.input_derivations.len(), 1);
    let (_, outputs) = nix_drv.input_derivations.iter().next().unwrap();
    assert_eq!(outputs.len(), 1);
    assert!(outputs.contains("dev"));
}

const WASM_COMPONENT_GENERATED_INPUT_COUNT: usize = 5;
const WASM_COMPONENT_BLAKE3_HEX_LENGTH: usize = 64;

#[derive(serde::Deserialize)]
struct WasmComponentGeneratedExport {
    generated_inputs: Vec<crunch_wasm_component_core::GeneratedInputCandidate>,
}

fn wasm_component_export_expression() -> String {
    let digest = "a".repeat(WASM_COMPONENT_BLAKE3_HEX_LENGTH);
    format!(
        r#"
        let mantle = import "lib.ncl" in
        let digest = "{digest}" in
        let object = fun name => {{
          logical_path = "/mantle/store/%{{name}}",
          digest_blake3 = digest,
          size_bytes = 1,
        }} in
        let tool = fun name => {{
          version = "test-version",
          executable = object name,
          configuration_identity_blake3 = digest,
        }} in
        let checked_manifest = {{
          name = "demo-component",
          package_resolution = {{
            default_registry = "wasi.dev",
            registries = [{{
              namespace = "wasi",
              registry = "wasi.dev",
              oci_registry = "ghcr.io",
              namespace_prefix = "webassembly/",
              credential_handle = "secret://registry/ghcr",
            }}],
            requirements = [{{
              package = "wasi:cli",
              requirement = "=0.2.0",
              registry = "wasi.dev",
              kind = 'wit,
            }}],
            local_overrides = [{{
              package = "wasi:cli",
              path = "/mantle/store/wasi-cli",
            }}],
          }},
          wit = {{ package = "demo:app", world = "demo", source = object "wit" }},
          implementation = {{
            source = object "source",
            package = "demo",
            crate_name = "demo",
          }},
          cohort = {{
            rust_toolchain = tool "rust",
            rust_target = "wasm32-wasip2",
            wkg = tool "wkg",
            wit_bindgen = tool "wit-bindgen",
            wasm_component_ld = tool "wasm-component-ld",
            wasm_tools = tool "wasm-tools",
            wac = tool "wac",
            wasi_virt = tool "wasi-virt",
            wizer = tool "wizer",
            wasmtime = tool "wasmtime",
          }},
          composition = {{
            package = "demo:composition",
            source_wac = "package demo:composition; export app.run;",
            nodes = [{{
              id = "app",
              package = "demo:app",
              world = "demo",
              artifact = object "app-component",
            }}],
            output_world = "demo",
          }},
          validation_profiles = {{
            octet_artifact_profile_identity_blake3 = digest,
            expected_runtime_profile_identity_blake3 = digest,
          }},
          outputs = [{{
            name = "component",
            path = "component.wasm",
            class = 'validated-portable-component,
          }}],
        }} | mantle.WasmComponentBuildManifest in
        mantle.exportWasmComponentBuild {{
          manifest = checked_manifest,
          source_identity_blake3 = digest,
          dependency_identity_blake3 = digest,
        }}
        "#
    )
}

#[test]
fn wasm_component_manifest_exports_owned_tool_inputs() {
    let expression = wasm_component_export_expression();
    let export: serde_json::Value =
        crunch_eval::evaluate_str_and_deserialize(&expression, &stdlib_import_path()).unwrap();
    let generated = export["generated_inputs"].as_array().unwrap();
    let serialized = serde_json::to_string(&export).unwrap();
    let typed: WasmComponentGeneratedExport =
        crunch_eval::evaluate_str_and_deserialize(&expression, &stdlib_import_path()).unwrap();
    let receipt_plan = crunch_wasm_component_core::finalize_generated_inputs(typed.generated_inputs);

    assert_eq!(export["schema"], "mantle-wasm-component-export-v1");
    assert_eq!(generated.len(), WASM_COMPONENT_GENERATED_INPUT_COUNT);
    assert!(generated.iter().all(|item| {
        item["owner"]["generator"] == "mantle-wasm-component-export"
            && item["owner"]["schema"] == "mantle-wasm-component-generated-input-receipt-v1"
            && item["owner"]["export_name"] == item["name"]
    }));
    assert!(serialized.contains("[namespace_registries]"));
    assert!(serialized.contains("[registry.\\\"wasi.dev\\\".oci]"));
    assert!(serialized.contains("protocol = \\\"https\\\""));
    assert!(serialized.contains("package demo:composition"));
    assert!(!serialized.contains("secret://registry/ghcr"));
    assert!(receipt_plan.blockers.is_empty());
    assert_eq!(receipt_plan.plan.unwrap().inputs.len(), WASM_COMPONENT_GENERATED_INPUT_COUNT);
}

#[test]
fn wasm_component_export_is_deterministic() {
    let expression = wasm_component_export_expression();
    let first: serde_json::Value =
        crunch_eval::evaluate_str_and_deserialize(&expression, &stdlib_import_path()).unwrap();
    let second: serde_json::Value =
        crunch_eval::evaluate_str_and_deserialize(&expression, &stdlib_import_path()).unwrap();

    assert_eq!(first, second);
    assert_eq!(first["source_identity_blake3"], "a".repeat(WASM_COMPONENT_BLAKE3_HEX_LENGTH));
}

#[test]
fn wasm_component_manifest_rejects_empty_world() {
    let valid = wasm_component_export_expression();
    let invalid = valid.replace("world = \"demo\", source", "world = \"\", source");
    assert_ne!(valid, invalid, "fixture mutation must change the WIT world");

    let result = crunch_eval::evaluate_str(&invalid, &stdlib_import_path());
    assert!(result.is_err(), "empty WIT worlds must fail the typed contract");
}

#[test]
fn wasm_component_export_rejects_digest_role_confusion() {
    let valid = wasm_component_export_expression();
    let invalid = valid.replace(
        "source_identity_blake3 = digest",
        "source_identity_blake3 = \"sha256:e7e85458e11caf76554b724ebf4f113259decf0f3b1ee2e2930de096f72114a7\"",
    );
    assert_ne!(valid, invalid, "fixture mutation must change the source identity role");

    let result = crunch_eval::evaluate_str(&invalid, &stdlib_import_path());
    assert!(result.is_err(), "OCI SHA-256 must not satisfy a Mantle BLAKE3 role");
}

#[test]
fn wasm_component_manifest_rejects_unknown_output_class() {
    let valid = wasm_component_export_expression();
    let invalid = valid.replace("class = 'validated-portable-component", "class = 'runtime-authority");
    assert_ne!(valid, invalid, "fixture mutation must change the output class");

    let result = crunch_eval::evaluate_str(&invalid, &stdlib_import_path());
    assert!(result.is_err(), "unknown output classes must fail the typed contract");
}
