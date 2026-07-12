#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
syn = { version = "=2.0.117", features = ["full", "parsing"] }
---

mod machine_schema_contracts;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use machine_schema_contracts::*;
use serde_json::Value;

const INVENTORY_PATH: &str = "schemas/machine-contracts/inventory.ncl";
const SOURCE_SCAN_ROOTS: &[&str] = &["src", "crates", "tools"];
const MAX_SOURCE_FILES: u32 = 4_096;
const BYTES_PER_KIBIBYTE: u64 = 1_024;
const KIBIBYTES_PER_MEBIBYTE: u64 = 1_024;
const MAX_SOURCE_FILE_MEBIBYTES: u64 = 2;
const MAX_CONTRACT_FILE_MEBIBYTES: u64 = 16;
const MAX_SOURCE_BYTES: u64 = MAX_SOURCE_FILE_MEBIBYTES * KIBIBYTES_PER_MEBIBYTE * BYTES_PER_KIBIBYTE;
const MAX_CONTRACT_FILE_BYTES: u64 = MAX_CONTRACT_FILE_MEBIBYTES * KIBIBYTES_PER_MEBIBYTE * BYTES_PER_KIBIBYTE;
const SELF_TEST_MAX_VALUES: u64 = 2;
const SELF_TEST_SCHEMA_PATH: &str = "schemas/machine-contracts/self-test.schema.json";
const SELF_TEST_CONTRACT_PATH: &str = "schemas/machine-contracts/self-test.contract.ncl";
const SELF_TEST_POSITIVE_PATH: &str = "schemas/machine-contracts/fixtures/self-test.valid.json";
const SELF_TEST_NEGATIVE_PATH: &str = "schemas/machine-contracts/fixtures/self-test.negatives.json";
const ZERO_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Check,
    Generate,
    SelfTest,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("machine schema contract check failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Vec<String>) -> Result<String, String> {
    let mode = parse_mode(&arguments)?;
    if mode == Mode::SelfTest {
        run_self_test()?;
        return Ok("machine schema contract self-test: PASS".to_string());
    }
    let root = env::current_dir().map_err(|err| format!("reading current directory: {err}"))?;
    require_repository_root(&root)?;
    let inventory_source = read_text(&root, INVENTORY_PATH)?;
    let mut registry = parse_registry_source(&inventory_source)?;
    reject_issues("registry", validate_registry(&registry))?;

    let sources = scan_rust_sources(&root)?;
    reject_issues("registered producer source paths", validate_registered_source_paths(&registry, &sources))?;
    let markers = extract_public_markers(&sources);
    reject_issues("public producer marker completeness", validate_marker_completeness(&registry, &markers))?;
    reject_issues("root JSON producer family coverage", validate_root_json_source_coverage(&registry, &sources))?;
    reject_issues("runtime Nickel contract boundary", validate_runtime_contract_boundary(&sources))?;

    let generated = render_and_validate_artifacts(&root, &registry)?;
    if mode == Mode::Generate {
        write_generated_contracts(&root, &generated)?;
        let files = load_referenced_files(&root, &registry)?;
        for surface in registry.surfaces.iter_mut().filter(|surface| surface.class == CONTRACTED_CLASS) {
            surface.freshness = expected_freshness(surface, &files)?;
        }
        let rendered_inventory = render_registry_source(&inventory_source, &registry)?;
        fs::write(root.join(INVENTORY_PATH), rendered_inventory)
            .map_err(|err| format!("writing {INVENTORY_PATH}: {err}"))?;
        return Ok(format!(
            "machine schema contract generation: PASS ({} contracted, {} classified)",
            registry.initial_cohort.len(),
            registry.surfaces.len()
        ));
    }

    compare_generated_contracts(&root, &generated)?;
    let files = load_referenced_files(&root, &registry)?;
    validate_contract_freshness(&registry, &files)?;
    validate_owner_shapes(&registry, &generated, &sources)?;
    Ok(format!(
        "machine schema contract check: PASS ({} contracted, {} classified)",
        registry.initial_cohort.len(),
        registry.surfaces.len()
    ))
}

fn parse_mode(arguments: &[String]) -> Result<Mode, String> {
    match arguments {
        [] => Ok(Mode::Check),
        [flag] if flag == "--generate" => Ok(Mode::Generate),
        [flag] if flag == "--self-test" => Ok(Mode::SelfTest),
        [flag] if flag == "--help" => {
            Err("usage: check-machine-schema-contracts.rs [--generate|--self-test]".to_string())
        }
        _ => Err("usage: check-machine-schema-contracts.rs [--generate|--self-test]".to_string()),
    }
}

fn require_repository_root(root: &Path) -> Result<(), String> {
    for required in ["Cargo.toml", INVENTORY_PATH, "schemas/machine-contracts"] {
        if !root.join(required).exists() {
            return Err(format!("run from repository root; missing {required}"));
        }
    }
    Ok(())
}

fn render_and_validate_artifacts(
    root: &Path,
    registry: &Registry,
) -> Result<BTreeMap<String, (Value, String)>, String> {
    let mut generated = BTreeMap::new();
    let mut observed_classes = BTreeSet::new();
    for surface in registry.surfaces.iter().filter(|surface| surface.class == CONTRACTED_CLASS) {
        let schema = read_json(root, &surface.artifacts.schema)?;
        reject_issues(&format!("{} schema", surface.id), validate_schema(&surface.id, &schema))?;
        let contract = render_contract(&surface.id, &schema)?;
        let positives = surface
            .artifacts
            .positive_fixtures
            .iter()
            .map(|path| Ok((path.clone(), read_json(root, path)?)))
            .collect::<Result<Vec<_>, String>>()?;
        let negatives =
            serde_json::from_value::<NegativeFixtureSet>(read_json(root, &surface.artifacts.negative_fixture_set)?)
                .map_err(|err| format!("parsing {}: {err}", surface.artifacts.negative_fixture_set))?;
        observed_classes.extend(negatives.cases.iter().map(|case| case.issue_class.clone()));
        reject_issues(
            &format!("{} fixtures", surface.id),
            validate_fixture_set(surface, &schema, &positives, &negatives),
        )?;
        generated.insert(surface.id.clone(), (schema, contract));
    }
    reject_issues("global adversarial fixture coverage", validate_global_fixture_coverage(&observed_classes))?;
    Ok(generated)
}

fn write_generated_contracts(root: &Path, generated: &BTreeMap<String, (Value, String)>) -> Result<(), String> {
    let registry = parse_registry_source(&read_text(root, INVENTORY_PATH)?)?;
    for surface in registry.surfaces.iter().filter(|surface| surface.class == CONTRACTED_CLASS) {
        let (_, contract) =
            generated.get(&surface.id).ok_or_else(|| format!("missing generated contract for {}", surface.id))?;
        fs::write(root.join(&surface.artifacts.generated_contract), contract)
            .map_err(|err| format!("writing {}: {err}", surface.artifacts.generated_contract))?;
    }
    Ok(())
}

fn compare_generated_contracts(root: &Path, generated: &BTreeMap<String, (Value, String)>) -> Result<(), String> {
    let registry = parse_registry_source(&read_text(root, INVENTORY_PATH)?)?;
    for surface in registry.surfaces.iter().filter(|surface| surface.class == CONTRACTED_CLASS) {
        let (_, expected) =
            generated.get(&surface.id).ok_or_else(|| format!("missing generated contract for {}", surface.id))?;
        let actual = read_text(root, &surface.artifacts.generated_contract)?;
        if actual != *expected {
            return Err(format!(
                "{} is stale; run `nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --generate`",
                surface.artifacts.generated_contract
            ));
        }
    }
    Ok(())
}

fn validate_contract_freshness(registry: &Registry, files: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    for surface in registry.surfaces.iter().filter(|surface| surface.class == CONTRACTED_CLASS) {
        let expected = expected_freshness(surface, files)?;
        reject_issues(&format!("{} freshness", surface.id), validate_freshness(surface, &expected))?;
    }
    Ok(())
}

fn validate_owner_shapes(
    registry: &Registry,
    generated: &BTreeMap<String, (Value, String)>,
    sources: &BTreeMap<String, String>,
) -> Result<(), String> {
    for surface in registry.surfaces.iter().filter(|surface| surface.class == CONTRACTED_CLASS) {
        let (schema, _) = generated.get(&surface.id).ok_or_else(|| format!("missing schema for {}", surface.id))?;
        reject_issues(
            &format!("{} Rust producer parity", surface.id),
            validate_rust_owner_shape(surface, schema, sources),
        )?;
    }
    Ok(())
}

fn load_referenced_files(root: &Path, registry: &Registry) -> Result<BTreeMap<String, Vec<u8>>, String> {
    all_referenced_paths(registry)
        .into_iter()
        .map(|path| {
            let bytes = fs::read(root.join(&path)).map_err(|err| format!("reading {path}: {err}"))?;
            Ok((path, bytes))
        })
        .collect()
}

fn scan_rust_sources(root: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut sources = BTreeMap::new();
    for &scan_root in SOURCE_SCAN_ROOTS {
        walk_rust_sources(root, &root.join(scan_root), &mut sources)?;
    }
    let source_count = u32::try_from(sources.len()).unwrap_or(u32::MAX);
    if source_count > MAX_SOURCE_FILES {
        return Err(format!("source scan exceeds MAX_SOURCE_FILES={MAX_SOURCE_FILES}"));
    }
    Ok(sources)
}

fn walk_rust_sources(
    repository_root: &Path,
    directory: &Path,
    sources: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let entries =
        fs::read_dir(directory).map_err(|err| format!("reading source directory {}: {err}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("reading source entry: {err}"))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|err| format!("reading type for {}: {err}", path.display()))?;
        if file_type.is_dir() {
            walk_rust_sources(repository_root, &path, sources)?;
            continue;
        }
        if !file_type.is_file() || path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let metadata = entry.metadata().map_err(|err| format!("reading metadata for {}: {err}", path.display()))?;
        if metadata.len() > MAX_SOURCE_BYTES {
            return Err(format!("source file {} exceeds MAX_SOURCE_BYTES={MAX_SOURCE_BYTES}", path.display()));
        }
        let relative = path
            .strip_prefix(repository_root)
            .map_err(|err| format!("normalizing {}: {err}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&path).map_err(|err| format!("reading source {}: {err}", path.display()))?;
        sources.insert(relative, source);
    }
    Ok(())
}

fn read_text(root: &Path, relative: &str) -> Result<String, String> {
    let path = root.join(relative);
    let byte_count = fs::metadata(&path).map_err(|err| format!("reading metadata for {relative}: {err}"))?.len();
    if byte_count > MAX_CONTRACT_FILE_BYTES {
        return Err(format!("{relative} exceeds MAX_CONTRACT_FILE_BYTES={MAX_CONTRACT_FILE_BYTES}"));
    }
    fs::read_to_string(path).map_err(|err| format!("reading {relative}: {err}"))
}

fn read_json(root: &Path, relative: &str) -> Result<Value, String> {
    serde_json::from_str(&read_text(root, relative)?).map_err(|err| format!("parsing JSON {relative}: {err}"))
}

fn reject_issues(context: &str, issues: Vec<Issue>) -> Result<(), String> {
    if issues.is_empty() {
        return Ok(());
    }
    let details = issues
        .into_iter()
        .map(|issue| format!("{} [{}] {}: {}", issue.surface_id, issue.class, issue.path, issue.message))
        .collect::<Vec<_>>()
        .join("\n  - ");
    Err(format!("{context} rejected:\n  - {details}"))
}

fn run_self_test() -> Result<(), String> {
    let schema = self_test_schema();
    reject_issues("self-test supported schema", validate_schema("self-test.surface", &schema))?;
    let valid = self_test_valid_instance();
    reject_issues("self-test valid instance", validate_instance("self-test.surface", &schema, &valid))?;
    let mut invalid = valid.clone();
    invalid["digest"] = Value::String("bad".to_string());
    invalid["values"] = serde_json::json!(["../escape", "b", "c"]);
    invalid["unknown"] = Value::Bool(true);
    let classes = validate_instance("self-test.surface", &schema, &invalid)
        .into_iter()
        .map(|issue| issue.class)
        .collect::<BTreeSet<_>>();
    for expected in ["digest", "reference", "bounds", "unknown-field", "cross-field"] {
        if !classes.contains(expected) {
            return Err(format!("self-test failed to detect {expected}: {classes:?}"));
        }
    }
    let mut unsupported = schema.clone();
    unsupported["oneOf"] = serde_json::json!([]);
    if validate_schema("self-test.surface", &unsupported).is_empty()
        || render_contract("self-test.surface", &unsupported).is_ok()
    {
        return Err("self-test accepted unsupported oneOf schema".to_string());
    }
    let rendered = render_contract("self-test.surface", &schema)?;
    for required in [
        "C.FromPredicate",
        "C.IsBlake3Hex",
        "C.IsSafeReference",
        "std.array.length",
    ] {
        if !rendered.contains(required) {
            return Err(format!("self-test rendered contract missed {required}"));
        }
    }
    let zero_digest_length = u32::try_from(ZERO_DIGEST.len()).unwrap_or(u32::MAX);
    let expected_bound = format!("let SELF_TEST_MAX_VALUES = {SELF_TEST_MAX_VALUES} in");
    if !rendered.contains(&expected_bound) {
        return Err("self-test rendered contract missed its named bound declaration".to_string());
    }
    if rendered.contains("Dyn") || zero_digest_length != BLAKE3_HEX_LENGTH_CHARS {
        return Err("self-test detected permissive rendering or digest-length drift".to_string());
    }
    run_utf8_bounds_self_test()?;
    run_schema_rejection_self_test()?;
    run_nullable_constraint_self_test()?;
    run_invariant_parity_self_test()?;
    run_rust_shape_self_test()?;
    run_registry_self_test()?;
    run_freshness_self_test()?;
    Ok(())
}

fn run_utf8_bounds_self_test() -> Result<(), String> {
    const UTF8_EXACT_BYTES: u64 = 4;
    let schema = serde_json::json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": "mantle://schemas/self-test-utf8-v1",
        "type": "string",
        "minLength": UTF8_EXACT_BYTES,
        "maxLength": UTF8_EXACT_BYTES,
        "x-mantle-length-unit": "utf8-bytes",
        "x-mantle-bound-name": {"minLength": "UTF8_EXACT_BYTES", "maxLength": "UTF8_EXACT_BYTES"}
    });
    reject_issues("self-test UTF-8 schema", validate_schema("self-test.utf8", &schema))?;
    reject_issues(
        "self-test exact UTF-8 byte length",
        validate_instance("self-test.utf8", &schema, &Value::String("éé".to_string())),
    )?;
    if validate_instance("self-test.utf8", &schema, &Value::String("ééé".to_string())).is_empty() {
        return Err("self-test accepted a UTF-8 string beyond the byte bound".to_string());
    }
    Ok(())
}

fn run_schema_rejection_self_test() -> Result<(), String> {
    const FRACTIONAL_ITEM_BOUND: f64 = 1.5;
    let mut invalid_union = self_test_schema();
    invalid_union["properties"]["digest"]["type"] = serde_json::json!(["string", "integer", "null"]);
    expect_schema_rejected("multi-type nullable union", &invalid_union)?;

    let mut ref_sibling = self_test_schema();
    ref_sibling["properties"]["digest"] = serde_json::json!({
        "$ref": "#/$defs/digest",
        "description": "ignored sibling",
    });
    ref_sibling["$defs"] = serde_json::json!({
        "digest": {"type": "string", "x-mantle-semantic": "blake3"}
    });
    expect_schema_rejected("reference sibling", &ref_sibling)?;

    let mut misplaced_bound = self_test_schema();
    misplaced_bound["properties"]["digest"]["maxItems"] = serde_json::json!(SELF_TEST_MAX_VALUES);
    misplaced_bound["properties"]["digest"]["x-mantle-bound-name"] =
        serde_json::json!({"maxItems": "SELF_TEST_MAX_VALUES"});
    expect_schema_rejected("type-incompatible bound", &misplaced_bound)?;

    let mut missing_items = self_test_schema();
    missing_items["properties"]["values"].as_object_mut().expect("array schema object").remove("items");
    expect_schema_rejected("untyped array", &missing_items)?;

    let mut fractional_bound = self_test_schema();
    fractional_bound["properties"]["values"]["maxItems"] = serde_json::json!(FRACTIONAL_ITEM_BOUND);
    expect_schema_rejected("fractional collection bound", &fractional_bound)?;

    let recursive = serde_json::json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": "mantle://schemas/self-test-recursive-v1",
        "$defs": {"loop": {"$ref": "#/$defs/loop"}},
        "type": "object",
        "required": ["loop"],
        "properties": {"loop": {"$ref": "#/$defs/loop"}},
        "additionalProperties": false
    });
    reject_issues("self-test recursive schema shape", validate_schema("self-test.recursive", &recursive))?;
    if render_contract("self-test.recursive", &recursive).is_ok() {
        return Err("self-test accepted a recursive contract reference".to_string());
    }
    Ok(())
}

fn expect_schema_rejected(label: &str, schema: &Value) -> Result<(), String> {
    if validate_schema("self-test.rejection", schema).is_empty() {
        return Err(format!("self-test accepted unsupported schema: {label}"));
    }
    if render_contract("self-test.rejection", schema).is_ok() {
        return Err(format!("self-test rendered unsupported schema: {label}"));
    }
    Ok(())
}

fn run_nullable_constraint_self_test() -> Result<(), String> {
    let schema = serde_json::json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": "mantle://schemas/self-test-nullable-const-v1",
        "type": ["string", "null"],
        "const": "exact"
    });
    reject_issues("self-test nullable const schema", validate_schema("self-test.nullable", &schema))?;
    if validate_instance("self-test.nullable", &schema, &Value::Null).is_empty() {
        return Err("self-test accepted null despite a non-null const constraint".to_string());
    }
    let rendered = render_contract("self-test.nullable", &schema)?;
    let const_offset =
        rendered.find("(value) == \"exact\"").ok_or_else(|| "nullable const was not rendered".to_string())?;
    let null_offset = rendered.find("(value) == null").ok_or_else(|| "nullable type was not rendered".to_string())?;
    if const_offset >= null_offset {
        return Err("nullable rendering bypasses its universal const constraint".to_string());
    }
    Ok(())
}

fn run_invariant_parity_self_test() -> Result<(), String> {
    let schema = serde_json::json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": "mantle://schemas/self-test-invariant-v1",
        "type": "object",
        "required": ["target"],
        "properties": {
            "target": {"type": "boolean"},
            "optional": {"type": "boolean"}
        },
        "additionalProperties": false,
        "x-mantle-invariants": [{"kind": "boolean-or", "terms": ["/optional"], "target": "/target"}]
    });
    reject_issues("self-test invariant schema", validate_schema("self-test.invariant", &schema))?;
    reject_issues(
        "self-test complete invariant value",
        validate_instance("self-test.invariant", &schema, &serde_json::json!({"target": false, "optional": false})),
    )?;
    let missing = validate_instance("self-test.invariant", &schema, &serde_json::json!({"target": false}));
    if !missing.iter().any(|issue| issue.class == "cross-field") {
        return Err(format!("self-test ignored a missing invariant operand: {missing:?}"));
    }

    let mut malformed = schema;
    malformed["x-mantle-invariants"] = serde_json::json!([{"kind": "boolean-or", "target": "/target"}]);
    expect_schema_rejected("invariant with empty terms", &malformed)?;
    Ok(())
}

fn run_rust_shape_self_test() -> Result<(), String> {
    let surface = self_test_surface();
    let schema = serde_json::json!({
        "x-mantle-rust-owner": surface.rust_owner.clone(),
        "type": "object",
        "required": ["wire"],
        "properties": {
            "wire": {"type": "string"},
            "optional": {"type": ["string", "null"]}
        },
        "additionalProperties": false
    });
    let source = r#"
        #[derive(serde::Serialize)]
        struct Report {
            #[serde(rename = "wire")]
            pub logical: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub optional: Option<String>,
            #[serde(skip_serializing)]
            pub hidden: String,
        }
    "#;
    let sources = BTreeMap::from([("src/self_test.rs".to_string(), source.to_string())]);
    reject_issues("self-test Rust owner shape", validate_rust_owner_shape(&surface, &schema, &sources))?;
    let mut stale = schema.clone();
    stale["required"] = serde_json::json!(["wire", "optional"]);
    if validate_rust_owner_shape(&surface, &stale, &sources).is_empty() {
        return Err("self-test missed Rust/schema optional-field drift".to_string());
    }
    let mut stale_type = schema.clone();
    stale_type["properties"]["wire"]["type"] = Value::String("boolean".to_string());
    if validate_rust_owner_shape(&surface, &stale_type, &sources).is_empty() {
        return Err("self-test missed Rust/schema scalar-type drift".to_string());
    }
    let spoofed = r#"
        // struct Report { pub wire: String }
        #[derive(serde::Serialize)]
        struct Report { pub wrong: String }
    "#;
    let spoofed_sources = BTreeMap::from([("src/self_test.rs".to_string(), spoofed.to_string())]);
    if validate_rust_owner_shape(&surface, &schema, &spoofed_sources).is_empty() {
        return Err("self-test let a commented struct spoof Rust owner parity".to_string());
    }
    let renamed = r#"
        #[derive(serde::Serialize)]
        #[serde(rename_all = "kebab-case")]
        struct Report { pub wire: String, pub optional: Option<String> }
    "#;
    let renamed_sources = BTreeMap::from([("src/self_test.rs".to_string(), renamed.to_string())]);
    if validate_rust_owner_shape(&surface, &schema, &renamed_sources).is_empty() {
        return Err("self-test accepted unsupported struct-level serializer rewriting".to_string());
    }
    Ok(())
}

fn self_test_schema() -> Value {
    serde_json::json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": "mantle://schemas/self-test-v1",
        "type": "object",
        "required": ["schema", "digest", "values", "count"],
        "properties": {
            "schema": {"type": "string", "const": "self-test-v1", "x-mantle-issue-class": "version"},
            "digest": {"type": "string", "x-mantle-semantic": "blake3"},
            "values": {
                "type": "array",
                "maxItems": SELF_TEST_MAX_VALUES,
                "x-mantle-bound-name": {"maxItems": "SELF_TEST_MAX_VALUES"},
                "items": {"type": "string", "x-mantle-semantic": "safe-reference"}
            },
            "count": {
                "type": "integer",
                "minimum": 0,
                "maximum": SELF_TEST_MAX_VALUES,
                "x-mantle-bound-name": {"minimum": "SELF_TEST_MIN_VALUES", "maximum": "SELF_TEST_MAX_VALUES"}
            }
        },
        "additionalProperties": false,
        "x-mantle-invariants": [{"kind": "length-equals", "array": "/values", "integer": "/count"}]
    })
}

fn self_test_valid_instance() -> Value {
    serde_json::json!({
        "schema": "self-test-v1",
        "digest": "a".repeat(BLAKE3_HEX_LENGTH_CHARS.try_into().expect("digest length fits host memory")),
        "values": ["artifact/reference"],
        "count": 1
    })
}

fn self_test_surface() -> Surface {
    Surface {
        id: "self-test.surface".to_string(),
        class: CONTRACTED_CLASS.to_string(),
        rust_owner: "src/self_test.rs::Report".to_string(),
        producer: Producer {
            command_or_api: "self-test producer".to_string(),
            marker: "self-test.surface".to_string(),
            source_paths: vec!["src/self_test.rs".to_string()],
        },
        consumers: vec!["self-test consumer".to_string()],
        artifacts: Artifacts {
            schema: SELF_TEST_SCHEMA_PATH.to_string(),
            generated_contract: SELF_TEST_CONTRACT_PATH.to_string(),
            positive_fixtures: vec![SELF_TEST_POSITIVE_PATH.to_string()],
            negative_fixture_set: SELF_TEST_NEGATIVE_PATH.to_string(),
        },
        version_policy: VersionPolicy {
            current: "self-test-v1".to_string(),
            supported: vec!["self-test-v1".to_string()],
            unknown_version_behavior: "reject".to_string(),
            compatibility_converter: String::new(),
            compatibility_fixtures: Vec::new(),
        },
        validation_commands: vec!["self-test".to_string()],
        fixture_coverage: vec!["schema".to_string()],
        freshness_strategy: "BLAKE3 self-test bindings".to_string(),
        freshness: Freshness {
            schema_blake3: ZERO_DIGEST.to_string(),
            contract_blake3: ZERO_DIGEST.to_string(),
            prelude_blake3: ZERO_DIGEST.to_string(),
            fixture_set_blake3: ZERO_DIGEST.to_string(),
            producer_identity_blake3: ZERO_DIGEST.to_string(),
            consumer_policy_blake3: ZERO_DIGEST.to_string(),
        },
        non_claims: vec!["self-test non-claim".to_string()],
        rationale: "self-test surface".to_string(),
    }
}

fn run_registry_self_test() -> Result<(), String> {
    let mut surface = self_test_surface();
    let registry = Registry {
        registry_schema: REGISTRY_SCHEMA.to_string(),
        prelude_path: PRELUDE_PATH.to_string(),
        supported_classes: SUPPORTED_SURFACE_CLASSES.iter().map(|class| (*class).to_string()).collect(),
        initial_cohort: vec![surface.id.clone()],
        surfaces: vec![surface.clone()],
    };
    reject_issues("self-test registry", validate_registry(&registry))?;
    let source_files = BTreeMap::from([("src/self_test.rs".to_string(), "struct Report {}".to_string())]);
    reject_issues("self-test registered source path", validate_registered_source_paths(&registry, &source_files))?;
    if validate_registered_source_paths(&registry, &BTreeMap::new()).is_empty() {
        return Err("self-test missed absent registered source path".to_string());
    }
    let discovered = BTreeMap::from([(surface.producer.marker.clone(), "src/self_test.rs".to_string())]);
    reject_issues("self-test complete marker set", validate_marker_completeness(&registry, &discovered))?;
    if validate_marker_completeness(&registry, &BTreeMap::new()).is_empty() {
        return Err("self-test missed absent public producer marker".to_string());
    }
    let extra = BTreeMap::from([
        (surface.producer.marker.clone(), "src/self_test.rs".to_string()),
        ("unclassified.surface".to_string(), "src/unclassified.rs".to_string()),
    ]);
    if validate_marker_completeness(&registry, &extra).is_empty() {
        return Err("self-test missed unclassified public producer".to_string());
    }
    let uncovered_source =
        BTreeMap::from([("src/unclassified.rs".to_string(), "serde_json::to_string_pretty(&report)".to_string())]);
    if validate_root_json_source_coverage(&registry, &uncovered_source).is_empty() {
        return Err("self-test missed unclassified root JSON serializer".to_string());
    }
    let runtime_contract_source =
        BTreeMap::from([("src/runtime.rs".to_string(), "let Contract = import \"surface.contract.ncl\"".to_string())]);
    if validate_runtime_contract_boundary(&runtime_contract_source).is_empty() {
        return Err("self-test missed runtime Nickel contract execution dependency".to_string());
    }
    let mut escaped_registry = registry.clone();
    escaped_registry.surfaces[0].artifacts.generated_contract = "../escaped.contract.ncl".to_string();
    if validate_registry(&escaped_registry).is_empty() {
        return Err("self-test accepted an artifact path outside the repository contract root".to_string());
    }
    let mut duplicate_registry = registry.clone();
    duplicate_registry.surfaces.push(surface.clone());
    duplicate_registry.initial_cohort.push(surface.id.clone());
    if validate_registry(&duplicate_registry).is_empty() {
        return Err("self-test missed duplicate ids and artifact ownership".to_string());
    }
    surface.version_policy.supported.push("self-test-v0".to_string());
    let mut migration_registry = registry;
    migration_registry.surfaces = vec![surface];
    if validate_registry(&migration_registry).is_empty() {
        return Err("self-test admitted prior version without converter fixtures".to_string());
    }
    Ok(())
}

fn run_freshness_self_test() -> Result<(), String> {
    let mut surface = self_test_surface();
    let mut files = BTreeMap::from([
        (PRELUDE_PATH.to_string(), b"prelude".to_vec()),
        (SELF_TEST_SCHEMA_PATH.to_string(), b"schema".to_vec()),
        (SELF_TEST_CONTRACT_PATH.to_string(), b"contract".to_vec()),
        (SELF_TEST_POSITIVE_PATH.to_string(), b"positive".to_vec()),
        (SELF_TEST_NEGATIVE_PATH.to_string(), b"negative".to_vec()),
        ("src/self_test.rs".to_string(), b"owner".to_vec()),
    ]);
    surface.freshness = expected_freshness(&surface, &files)?;
    reject_issues("self-test current freshness", validate_freshness(&surface, &expected_freshness(&surface, &files)?))?;
    for path in [
        SELF_TEST_SCHEMA_PATH,
        SELF_TEST_CONTRACT_PATH,
        SELF_TEST_POSITIVE_PATH,
        SELF_TEST_NEGATIVE_PATH,
        "src/self_test.rs",
    ] {
        let original = files.get(path).cloned().ok_or_else(|| format!("self-test file missing: {path}"))?;
        files.insert(path.to_string(), b"mutated".to_vec());
        if validate_freshness(&surface, &expected_freshness(&surface, &files)?).is_empty() {
            return Err(format!("self-test missed stale freshness input {path}"));
        }
        files.insert(path.to_string(), original);
    }
    validate_producer_freshness_mutations(&surface, &files)?;
    validate_policy_freshness_mutations(&surface, &files)?;
    Ok(())
}

fn validate_producer_freshness_mutations(surface: &Surface, files: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    for mutation in ["rust-owner", "producer-marker", "producer-command"] {
        let mut changed = surface.clone();
        match mutation {
            "rust-owner" => changed.rust_owner.push_str("Changed"),
            "producer-marker" => changed.producer.marker.push_str(".changed"),
            "producer-command" => changed.producer.command_or_api.push_str(" changed"),
            _ => return Err(format!("unknown producer freshness mutation {mutation}")),
        }
        if validate_freshness(surface, &expected_freshness(&changed, files)?).is_empty() {
            return Err(format!("self-test missed stale {mutation} identity"));
        }
    }
    Ok(())
}

fn validate_policy_freshness_mutations(surface: &Surface, files: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    for mutation in ["consumer", "non-claim"] {
        let mut changed = surface.clone();
        if mutation == "consumer" {
            changed.consumers.push("new consumer".to_string());
        } else {
            changed.non_claims.push("new non-claim".to_string());
        }
        if validate_freshness(surface, &expected_freshness(&changed, files)?).is_empty() {
            return Err(format!("self-test missed stale {mutation} policy"));
        }
    }
    Ok(())
}
