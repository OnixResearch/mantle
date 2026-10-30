use std::fs;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use crunch_wasm_component::ComponentPipelineRequest;
use crunch_wasm_component::ToolchainManifest;
use crunch_wasm_component::copy_source_tree;
use crunch_wasm_component::verify_toolchain_manifest;
use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::GENERATED_INPUT_OWNER;
use crunch_wasm_component_core::GENERATED_INPUT_RECEIPT_SCHEMA;
use serde_json::Value;
use serde_json::json;

type AuthorResult<T> = Result<T, Box<dyn std::error::Error>>;

const AUTHOR_ARGUMENT_COUNT: usize = 7;
const MAX_MEMBER_BYTES: u64 = 536_870_912;
const READ_BUFFER_BYTES: usize = 65_536;
const COHORT_TOOLS: [(&str, &str); 9] = [
    ("rust_toolchain", "rustc"),
    ("wkg", "wkg"),
    ("wit_bindgen", "wit-bindgen"),
    ("wasm_component_ld", "wasm-component-ld"),
    ("wasm_tools", "wasm-tools"),
    ("wac", "wac"),
    ("wasi_virt", "wasi-virt"),
    ("wizer", "wizer"),
    ("wasmtime", "wasmtime"),
];

// r[impl materialize_wrpc_producer.component_pair]
// r[impl materialize_wrpc_producer.consumer_contract]
fn main() -> AuthorResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != AUTHOR_ARGUMENT_COUNT {
        return Err(usage());
    }
    if !matches!(args[1].as_str(), "client" | "server" | "telemetry") {
        return Err(usage());
    }
    let typed: ComponentPipelineRequest = serde_json::from_value(request(&args)?)?;
    let serialized = serde_json::to_string(&typed)?;
    assert_eq!(args.len(), AUTHOR_ARGUMENT_COUNT);
    assert!(!serialized.is_empty());
    println!("std.deserialize 'Json {}", serde_json::to_string(&serialized)?);
    Ok(())
}

fn usage() -> Box<dyn std::error::Error> {
    "usage: mantle-wrpc-author ROLE TOOLCHAIN SOURCE REGISTRY COMPILED SERVER-OR-DASH".into()
}

fn request(args: &[String]) -> AuthorResult<Value> {
    assert_eq!(args.len(), AUTHOR_ARGUMENT_COUNT);
    let role = args[1].as_str();
    let toolchain_root = Path::new(&args[2]);
    let toolchain_path = toolchain_root.join("share/mantle/wasm-component-toolchain.json");
    let toolchain = verify_toolchain_manifest(&toolchain_path)?;
    let source = Path::new(&args[3]);
    let scratch = tempfile::tempdir()?;
    let source_object = copy_source_tree(source, &scratch.path().join("source"))?;
    let profile_text = fs::read_to_string(source.join(format!("{role}/runtime-profile-ref.txt")))?;
    let profile_identity = Blake3Identity::parse(profile_text.trim().to_string()).map_err(|error| error.to_string())?;
    let facts = json!({
        "source": source_object, "compiled": object(Path::new(&args[5]))?,
        "wit": object(&source.join(format!("{role}/wit/world.wit")))?,
        "profile": profile_identity,
        "registry": args[4], "tools": cohort(toolchain_root, &toolchain.manifest)?
    });
    let mut request = base_request(role, manifest(role, &toolchain.manifest, facts)?, &toolchain_path, source);
    if role == "client" {
        configure_client(&mut request, Path::new(&args[6]))?;
    }
    if role == "telemetry" {
        configure_telemetry(&mut request, Path::new(&args[6]))?;
    }
    request["generated_inputs"] = generated(&request)?;
    debug_assert!(request["generated_inputs"].is_array());
    Ok(request)
}

fn base_request(role: &str, manifest: Value, toolchain_path: &Path, source: &Path) -> Value {
    debug_assert!(!role.is_empty());
    debug_assert!(manifest.is_object());
    json!({
        "schema": "mantle-wasm-component-pipeline-request-v1", "manifest": manifest,
        "generated_inputs": [], "toolchain_manifest": toolchain_path,
        "source_root": source, "wit_relative_path": format!("{role}/wit"),
        "package_materializations": [],
        "cargo_component_relative_path": format!("wasm32-wasip2/release/mantle_wrpc_{role}.wasm"),
        "cargo_core_module_relative_path": null,
        "composition_dependencies": [{"package": "root:component", "source": "compiled-component"}],
        "package_resolution_network": false, "runtime_composition": null,
        "runtime_invoke": "run()", "expected_runtime_stdout": "42\n",
        "aot_target": "", "aot_cpu_features": [], "aot_configuration_args": []
    })
}

// r[impl materialize_wrpc_producer.runtime_binding]
fn configure_client(request: &mut Value, server: &Path) -> AuthorResult<()> {
    let server_object = object(server)?;
    debug_assert!(server.is_absolute());
    debug_assert!(request.is_object());
    request["runtime_composition"] = json!({
        "source_wac": "package mantle:rpc-smoke;\nlet server = new pilot:server {};\nlet client = new pilot:client { \"wrpc-examples:hello/handler\": server[\"wrpc-examples:hello/handler\"] };\nexport client.run;\n",
        "dependencies": [
            {"package":"pilot:client", "source":"compiled-component"},
            {"package":"pilot:server", "source":{"store-object":{
                "path":server, "digest_blake3":server_object["digest_blake3"]
            }}}
        ]
    });
    Ok(())
}

fn object(path: &Path) -> AuthorResult<Value> {
    let metadata = fs::symlink_metadata(path)?;
    let size_bytes = metadata.len();
    if !path.is_absolute() {
        return Err("producer members must be absolute paths".into());
    }
    if !metadata.is_file() {
        return Err("producer members must be regular files".into());
    }
    if size_bytes == 0 || size_bytes > MAX_MEMBER_BYTES {
        return Err("producer members must be bounded nonempty files".into());
    }
    let file = fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut reader = file.take(size_bytes.saturating_add(1));
    let mut buffer = [0_u8; READ_BUFFER_BYTES];
    let mut actual_bytes = 0_u64;
    let reads_max = size_bytes.div_ceil(u64::try_from(READ_BUFFER_BYTES)?).saturating_add(1);
    for _ in 0..=reads_max {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        actual_bytes = actual_bytes.checked_add(u64::try_from(count)?).ok_or("producer member size overflow")?;
    }
    if actual_bytes != size_bytes {
        return Err("producer member size changed during measurement".into());
    }
    debug_assert!(size_bytes > 0);
    debug_assert_eq!(actual_bytes, size_bytes);
    Ok(json!({"logical_path":path, "digest_blake3":hasher.finalize().to_hex().as_str(), "size_bytes":size_bytes}))
}

fn cohort(root: &Path, tools: &ToolchainManifest) -> AuthorResult<Value> {
    let mut values = COHORT_TOOLS
        .iter()
        .map(|(key, name)| {
            let tool = tools.tools.iter().find(|tool| tool.name == *name).ok_or("missing tool")?;
            let fact = json!({
                "version": tool.version, "executable":object(&root.join(&tool.path))?,
                "configuration_identity_blake3":tool.binary_digest_blake3
            });
            Ok(((*key).to_string(), fact))
        })
        .collect::<AuthorResult<serde_json::Map<String, Value>>>()?;
    values.insert("rust_target".to_string(), json!("wasm32-wasip2"));
    assert_eq!(values.len(), COHORT_TOOLS.len().saturating_add(1));
    assert!(values.contains_key("rust_toolchain"));
    Ok(Value::Object(values))
}

/// The client alone keeps its declared non-WASI hello import open for remoting.
fn remaining_imports(role: &str) -> Vec<&'static str> {
    debug_assert!(!role.is_empty());
    if role == "client" {
        return vec!["wrpc-examples:hello/handler"];
    }
    debug_assert_ne!(role, "client");
    Vec::new()
}

fn export_statements(role: &str) -> &'static str {
    debug_assert!(!role.is_empty());
    if role == "server" {
        return "export app[\"wrpc-examples:hello/handler\"];\nexport app.run;\n";
    }
    debug_assert_ne!(role, "server");
    "export app.run;\n"
}

fn manifest(role: &str, toolchain: &ToolchainManifest, facts: Value) -> AuthorResult<Value> {
    debug_assert!(facts.is_object());
    debug_assert!(!role.is_empty());
    Ok(json!({
        "schema":crunch_wasm_component_core::COMPONENT_MANIFEST_SCHEMA, "name":format!("mantle-wrpc-{role}"),
        "package_resolution":{
            "default_registry":"local", "registries":[{"namespace":"wrpc-examples", "registry":"local",
                "backend":"local", "oci_registry":null, "namespace_prefix":"", "protocol":"https",
                "local_root":facts["registry"], "credential_handle":null}],
            "requirements":[], "local_overrides":[], "lock_path":"wkg.lock"
        },
        "wit":{"package":"mantle:wrpc-pilot", "world":role, "source":facts["wit"]},
        "implementation":{"source":facts["source"], "package":format!("mantle-wrpc-{role}"),
            "crate_name":format!("mantle_wrpc_{role}"), "features":[], "profile":"release"},
        "cohort":facts["tools"],
        "composition":{"package":"mantle:wrpc", "source_wac":format!("package mantle:wrpc;\nlet app = new root:component {{ ... }};\n{}", export_statements(role)),
            "nodes":[{"id":"app", "package":"root:component", "world":role,
                "artifact":facts["compiled"], "required_imports":[]}],
            "edges":[], "output_world":role, "import_dependencies":false},
        "virtualization":{"defaults_overridden":true, "rules":[], "expected_remaining_imports":remaining_imports(role)},
        "validation_profiles":{"octet_artifact_profile_identity_blake3":toolchain.octet.profile_identity_blake3,
            "expected_runtime_profile_identity_blake3":facts["profile"]},
        "wizer":{"mode":"disabled", "initialization_entrypoint":null, "deterministic_virtual_imports":[],
            "linker_split":null, "componentization":null},
        "aot":{"mode":"disabled", "target":null, "cpu_features":[], "wasmtime_configuration_identity_blake3":null},
        "outputs":[{"name":"component", "path":"component.wasm", "class":"validated-portable-component"}],
        "non_claims":["not-component-behavior-correctness", "not-runtime-authority", "not-runtime-sandboxing",
            "not-release-eligibility", "not-octet-policy-interpretation", "not-remote-rpc-execution"]
    }))
}

// r[impl materialize_wrpc_telemetry.producer]
fn configure_telemetry(request: &mut Value, sink: &Path) -> AuthorResult<()> {
    debug_assert!(sink.is_absolute());
    debug_assert!(request.is_object());
    request["manifest"]["wit"]["package"] = json!("mantle:wrpc-telemetry-pilot");
    request["manifest"]["wit"]["world"] = json!("emitter");
    request["manifest"]["composition"]["nodes"][0]["world"] = json!("emitter");
    request["manifest"]["composition"]["output_world"] = json!("emitter");
    request["manifest"]["package_resolution"]["registries"][0]["namespace"] = json!("wasi");
    request["manifest"]["virtualization"]["expected_remaining_imports"] = json!(["wasi:otel/tracing@0.2.0-rc.2"]);
    request["runtime_invoke"] = json!("run(0)");
    request["runtime_composition"] = json!({
        "source_wac": "package mantle:otel-smoke;\nlet sink = new pilot:sink { ... };\nlet emitter = new pilot:emitter { \"wasi:otel/tracing@0.2.0-rc.2\": sink[\"wasi:otel/tracing@0.2.0-rc.2\"], ... };\nexport emitter.run;\n",
        "dependencies": [
            {"package":"pilot:emitter", "source":"compiled-component"},
            {"package":"pilot:sink", "source":{"store-object":{
                "path":sink, "digest_blake3":object(sink)?["digest_blake3"]
            }}}
        ]
    });
    Ok(())
}

fn generated(request: &Value) -> AuthorResult<Value> {
    let manifest = &request["manifest"];
    let registry = &manifest["package_resolution"]["registries"][0]["local_root"];
    let namespace =
        manifest["package_resolution"]["registries"][0]["namespace"].as_str().ok_or("registry namespace")?;
    let config = format!(
        "default_registry = \"local\"\n[namespace_registries]\n\"{namespace}\" = \"local\"\n[registry.\"local\"]\ndefault = \"local\"\n[registry.\"local\".local]\nroot = {registry}\n"
    );
    let candidates: Vec<Value> = [
        ("wkg-manifest", "wkg.toml", "tool-input", "[overrides]\n".to_string()),
        ("wkg-config", "wkg-config.toml", "tool-input", config),
        ("wac-source", "composition.wac", "composition-input", manifest["composition"]["source_wac"].as_str().ok_or("WAC source")?.to_string()),
        ("command-plan", "command-plan.json", "tool-input", serde_json::to_string(&request["runtime_composition"])?),
        ("runtime-profile", "runtime-profile.json", "runtime-profile", serde_json::to_string(&manifest["validation_profiles"])?),
    ].into_iter().map(|(name, target, class, content)| json!({
        "name":name, "target":format!(".mantle/wasm-component/{target}"), "output_class":class,
        "owner":{"schema":GENERATED_INPUT_RECEIPT_SCHEMA, "generator":GENERATED_INPUT_OWNER, "export_name":name,
            "source_identity_blake3":manifest["implementation"]["source"]["digest_blake3"],
            "dependency_identity_blake3":manifest["validation_profiles"]["expected_runtime_profile_identity_blake3"]},
        "content":content
    })).collect();
    assert_eq!(candidates.len(), 5);
    assert!(!namespace.is_empty());
    Ok(json!(candidates))
}
