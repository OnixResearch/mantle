//! Operator-local endpoint bindings for remote build and substitution tests.
//!
//! Remote production clients, `local-build` receivers and HTTP substituter
//! reads refuse to proceed without `--remote-binding-config`. This fixture
//! writes active bindings into the private table under
//! `<endpoint state>/remote-builders`, the table `mantle remote live serve`
//! maintains, and the private config file that names those exact bindings.
//! Every consumer loads the file and re-resolves each binding against the table
//! before any payload work.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use crunch_remote_core::endpoint::EndpointBinding;
use crunch_remote_core::endpoint::EndpointBindingRecord;
use crunch_remote_core::endpoint::EndpointBindingSpec;
use crunch_remote_core::endpoint::EndpointKind;

const CONFIG_SCHEMA: &str = "mantle-remote-endpoint-binding-config-v1";
const TABLE_DIRECTORY: &str = "remote-builders";
const TABLE_FILENAME: &str = "endpoint-bindings.json";
const TABLE_SCHEMA_VERSION: u32 = 1;
const SCOPE: &str = "remote-build-test";
const TRUST_REQUIREMENT: &str = "signed-builder-output";
const BINDING_TTL_SECS: u64 = 3_600;
const FIRST_GENERATION: u64 = 1;
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;

/// The endpoints one fixture binds under a single store prefix.
pub struct Bindings<'a> {
    pub store_prefix: &'a str,
    /// Builder-ticket endpoint name admitted for remote dispatch and serving.
    pub builder: Option<&'a str>,
    /// HTTP binary-cache URLs admitted as substituter sources.
    pub substituters: &'a [&'a str],
}

/// Write the binding table below `endpoint_state_dir` and the config that
/// admits exactly those bindings to `config_path`.
pub fn write_endpoint_bindings(endpoint_state_dir: &Path, config_path: &Path, bindings: Bindings<'_>) {
    let now_unix_s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let expires_unix_s = now_unix_s.checked_add(BINDING_TTL_SECS).unwrap();
    let binding = |kind, name: String| {
        let binding = EndpointBinding {
            spec: EndpointBindingSpec {
                kind,
                name,
                scope: SCOPE.to_string(),
                trust_requirement: TRUST_REQUIREMENT.to_string(),
                store_prefix: bindings.store_prefix.to_string(),
                expires_unix_s,
            },
            generation: FIRST_GENERATION,
        };
        binding.validate().expect("valid endpoint binding");
        binding
    };
    let builder = bindings.builder.map(|name| binding(EndpointKind::BuilderTicket, name.to_string()));
    let sources = bindings
        .substituters
        .iter()
        .enumerate()
        .map(|(index, url)| {
            // Admission matches the canonical URL the store derives from its input.
            let canonical = url::Url::parse(url).expect("substituter URL").as_str().to_string();
            (canonical, binding(EndpointKind::Substituter, format!("substituter-{index}")))
        })
        .collect::<Vec<_>>();
    let records = builder
        .iter()
        .chain(sources.iter().map(|(_, binding)| binding))
        .map(|binding| EndpointBindingRecord {
            binding: binding.clone(),
            revoked: false,
        })
        .collect::<Vec<_>>();
    let table_dir = endpoint_state_dir.join(TABLE_DIRECTORY);
    fs::create_dir_all(&table_dir).expect("endpoint binding table directory");
    fs::set_permissions(&table_dir, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE))
        .expect("private endpoint binding table directory");
    let table = serde_json::json!({ "schema_version": TABLE_SCHEMA_VERSION, "records": records });
    write_private_json(&table_dir.join(TABLE_FILENAME), &table);
    let build = builder.map(|binding| {
        serde_json::json!({ "endpoint_binding": binding, "scope": SCOPE, "trust_requirement": TRUST_REQUIREMENT })
    });
    let source_bindings = sources
        .into_iter()
        .map(|(source, binding)| {
            serde_json::json!({
                "source": source,
                "endpoint_binding": binding,
                "scope": SCOPE,
                "trust_requirement": TRUST_REQUIREMENT,
            })
        })
        .collect::<Vec<_>>();
    let config = serde_json::json!({
        "schema": CONFIG_SCHEMA,
        "endpoint_state_directory": endpoint_state_dir,
        "build": build,
        "source_bindings": source_bindings,
    });
    write_private_json(config_path, &config);
}

fn write_private_json(path: &Path, value: &serde_json::Value) {
    fs::write(path, serde_json::to_vec_pretty(value).expect("endpoint binding JSON")).expect("write binding JSON");
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_FILE_MODE)).expect("private endpoint binding JSON");
}
