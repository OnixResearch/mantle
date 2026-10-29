use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crunch_wasm_component::ComponentPipelineRequest;
use crunch_wasm_component::OCTET_MOLTEN_EFFECTS_PROFILE_ID;
use crunch_wasm_component::copy_source_tree;
use crunch_wasm_component::verify_toolchain_manifest;
use crunch_wasm_component::write_json_new;
use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::StoreObject;
use serde_json::Value;
use serde_json::json;

// r[impl mantle.wasm_component.effect_fixtures]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(4..=5).contains(&args.len()) {
        return Err("expected absolute toolchain root, new output directory, exact Molten WIT file, runtime profile BLAKE3, optional explicit guest source directory".into());
    }
    let toolchain_root = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[1]);
    if !toolchain_root.is_absolute() || !output.is_absolute() {
        return Err("absolute paths required".into());
    }
    let runtime_profile = Blake3Identity::parse(args[3].to_str().ok_or("runtime profile is not UTF-8")?.to_owned())
        .map_err(|error| format!("invalid runtime profile identity: {error}"))?;
    let toolchain_path = toolchain_root.join("share/mantle/wasm-component-toolchain.json");
    let toolchain = verify_toolchain_manifest(&toolchain_path)?;
    let effects = toolchain
        .manifest
        .octet
        .profiles
        .iter()
        .find(|profile| profile.profile_id == OCTET_MOLTEN_EFFECTS_PROFILE_ID)
        .ok_or("closed toolchain has no effect-world profile")?;
    let fixture = match args.get(4) {
        Some(source) => PathBuf::from(source),
        None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or("component example must have a workspace ancestor")?
            .join("fixtures/wasm-component-effects"),
    };
    let cargo: toml::Value = toml::from_str(&fs::read_to_string(fixture.join("Cargo.toml"))?)?;
    let package = cargo
        .get("package")
        .and_then(|value| value.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or("guest Cargo.toml needs package.name")?;
    let crate_name = package.replace('-', "_");
    if fs::read(fixture.join("wit/runtime.wit"))? != fs::read(&args[2])? {
        return Err("Molten-owned exact WIT bytes differ from reviewed producer fixture".into());
    }
    fs::create_dir(&output)?;
    let source = output.join("source");
    copy_source_tree(&fixture, &source)?;
    let generation = Command::new(toolchain.tool_path("wit-bindgen")?)
        .args([
            "rust",
            "--world",
            "actor-effects",
            "--generate-all",
            "--disable-run-ctors-once-workaround",
            "--disable-custom-section-link-helpers",
            "--out-dir",
        ])
        .arg(source.join("src"))
        .arg(source.join("wit"))
        .env_clear()
        .output()?;
    if !generation.status.success() {
        return Err(format!("binding generation failed: {}", String::from_utf8_lossy(&generation.stderr)).into());
    }
    write_json_new(
        &output.join("binding-generation.json"),
        &json!({
            "program": toolchain.tool_path("wit-bindgen")?, "program_blake3": toolchain.tool_digest("wit-bindgen")?,
            "world": "actor-effects", "flags": ["--generate-all", "--disable-run-ctors-once-workaround", "--disable-custom-section-link-helpers"],
            "bindings": object(&source.join("src/actor_effects.rs"))?
        }),
    )?;
    let hash_copy = output.join("source-identity-copy");
    let source_object = copy_source_tree(&source, &hash_copy)?;
    fs::remove_dir_all(hash_copy)?;
    let registry = output.join("registry");
    fs::create_dir(&registry)?;
    let wkg_config = format!(
        "default_registry = \"local\"\n\n[namespace_registries]\n\"molten\" = \"local\"\n\n[registry.\"local\"]\ndefault = \"local\"\n[registry.\"local\".local]\nroot = {}\n",
        serde_json::to_string(&registry)?
    );
    let composition = "package fixture:effects;\nlet app = new root:component { ... };\nexport app.invoke;\n";
    let mut cohort = BTreeMap::new();
    for (field, name) in [
        ("rust_toolchain", "rustc"),
        ("wkg", "wkg"),
        ("wit_bindgen", "wit-bindgen"),
        ("wasm_component_ld", "wasm-component-ld"),
        ("wasm_tools", "wasm-tools"),
        ("wac", "wac"),
        ("wasi_virt", "wasi-virt"),
        ("wizer", "wizer"),
        ("wasmtime", "wasmtime"),
    ] {
        let record = toolchain.manifest.tools.iter().find(|record| record.name == name).ok_or("missing pinned tool")?;
        cohort.insert(field, json!({"version": record.version, "executable": object(&toolchain.tool_path(name)?)?, "configuration_identity_blake3": record.binary_digest_blake3}));
    }
    cohort.insert("rust_target", json!("wasm32-wasip2"));
    let source_identity = source_object.digest_blake3.clone();
    let generated_inputs: Vec<Value> = [
        ("wkg-manifest", ".mantle/wasm-component/wkg.toml", "tool-input", "[overrides]\n".to_owned()),
        ("wkg-config", ".mantle/wasm-component/wkg-config.toml", "tool-input", wkg_config),
        ("wac-source", ".mantle/wasm-component/composition.wac", "composition-input", composition.to_owned()),
        ("command-plan", ".mantle/wasm-component/command-plan.json", "tool-input", "{}".to_owned()),
        (
            "runtime-profile",
            ".mantle/wasm-component/runtime-profile.json",
            "runtime-profile",
            serde_json::to_string(&json!({"required_runtime_profile_blake3": runtime_profile}))?,
        ),
    ]
    .into_iter()
    .map(|(name, target, class, content)| {
        json!({
            "name": name, "target": target, "output_class": class, "content": content,
            "owner": {"schema": crunch_wasm_component_core::GENERATED_INPUT_RECEIPT_SCHEMA,
                "generator": crunch_wasm_component_core::GENERATED_INPUT_OWNER, "export_name": name,
                "source_identity_blake3": source_identity, "dependency_identity_blake3": source_identity}
        })
    })
    .collect();
    let request: ComponentPipelineRequest = serde_json::from_value(json!({
        "schema": crunch_wasm_component::PIPELINE_REQUEST_SCHEMA,
        "manifest": {
            "schema": crunch_wasm_component_core::COMPONENT_MANIFEST_SCHEMA,
            "name": package,
            "package_resolution": {"default_registry": "local", "registries": [{"namespace": "molten", "registry": "local", "backend": "local", "oci_registry": null, "namespace_prefix": "", "protocol": "https", "local_root": registry, "credential_handle": null}], "requirements": [], "local_overrides": [], "lock_path": "wkg.lock"},
            "wit": {"package": "molten:component-effects", "world": "actor-effects", "source": object(&source.join("wit/runtime.wit"))?},
            "implementation": {"source": source_object, "package": package, "crate_name": crate_name, "features": [], "profile": "release"},
            "cohort": cohort,
            "composition": {"package": "fixture:effects", "source_wac": composition, "nodes": [{"id": "app", "package": "root:component", "world": "actor-effects", "artifact": {"logical_path": "/mantle/pending/compiled.wasm", "digest_blake3": Blake3Identity::from_slice(b"pending-component"), "size_bytes": 1}, "required_imports": []}], "edges": [], "output_world": "actor-effects", "import_dependencies": false},
            "virtualization": {"defaults_overridden": true, "rules": [], "expected_remaining_imports": ["effect-request"]},
            "validation_profiles": {"octet_artifact_profile_identity_blake3": effects.profile_identity_blake3, "expected_runtime_profile_identity_blake3": runtime_profile},
            "wizer": {"mode": "disabled", "initialization_entrypoint": null, "deterministic_virtual_imports": [], "linker_split": null, "componentization": null},
            "aot": {"mode": "disabled", "target": null, "cpu_features": [], "wasmtime_configuration_identity_blake3": null},
            "outputs": [{"name": "component", "path": "component.wasm", "class": "validated-portable-component"}],
            "non_claims": ["not-component-behavior-correctness", "not-runtime-authority", "not-runtime-sandboxing", "not-release-eligibility", "not-octet-policy-interpretation", "not-consumer-host-execution"]
        },
        "generated_inputs": generated_inputs, "toolchain_manifest": toolchain_path, "source_root": source,
        "wit_relative_path": "wit", "package_materializations": [],
        "cargo_component_relative_path": format!("wasm32-wasip2/release/{crate_name}.wasm"), "cargo_core_module_relative_path": null,
        "composition_dependencies": [{"package": "root:component", "source": "compiled-component"}],
        "package_resolution_network": false,
        "runtime_validation": {"mode": "consumer-host-required", "wit_package": "molten:component-effects", "wit_world": "actor-effects", "required_imports": ["effect-request"]},
        "runtime_invoke": null, "expected_runtime_stdout": "", "aot_target": "x86_64-unknown-linux-gnu", "aot_cpu_features": [], "aot_configuration_args": []
    }))?;
    write_json_new(&output.join("request.json"), &request)?;
    let nickel_json = serde_json::to_string(&serde_json::to_string(&request)?)?;
    fs::write(output.join("request.ncl"), format!("std.deserialize 'Json {nickel_json}\n"))?;
    println!("{}", output.join("request.ncl").display());
    Ok(())
}

fn object(path: &Path) -> Result<StoreObject, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    Ok(StoreObject {
        logical_path: path.display().to_string(),
        digest_blake3: Blake3Identity::from_slice(&bytes),
        size_bytes: bytes.len().try_into()?,
    })
}
