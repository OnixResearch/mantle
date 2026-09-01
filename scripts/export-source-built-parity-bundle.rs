#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
zstd = "=0.13.3"
---

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

const BUNDLE_SCHEMA: &str = "mantle-full-bootstrap-parity-bundle-v1";
const MEMBER_ENCODING: &str = "zstd-json-v1";
const RELEASE_ID: &str = "source-built-fixed-point-v98-20260831";
const PROVIDER_KIND: &str = "full-source";
const DESCRIPTOR_PATH: &str = "bootstrap/evidence/real-self-build-proof-parity.json";
const PROVIDER_LINKAGE_PATH: &str = "bootstrap/evidence/crunch-self-build-provider-kind-linkage.json";
const STAGEX_RECEIPT_PATH: &str = "bootstrap/evidence/stagex-lineage-provider-receipt.json";
const ROW_RECEIPT_PATHS: &[(&str, &str)] = &[
    ("row-early-binutils", "bootstrap/evidence/early-native-binutils-row-v2.json"),
    ("row-early-gcc40", "bootstrap/evidence/early-native-gcc40-row-v2.json"),
    ("row-final-gcc47", "bootstrap/evidence/final-native-gcc47-row-v1.json"),
    ("row-final-gcc10", "bootstrap/evidence/final-native-gcc10-row-v1.json"),
    ("row-final-musl-binutils", "bootstrap/evidence/final-native-musl-binutils-row-v1.json"),
];
const RUST_PROVIDER_ACTION_ROOT: &str = "rust-provider/share/mantle-rust-provider/action-trust";
const RUST_STAGE_NAMES: &[&str] = &[
    "mrustc-to-rust-1_90_0",
    "rust-1_91_1-stage1",
    "rust-1_92_0-stage1",
    "rust-1_93_1-stage1",
    "rust-1_94_0-final",
];
const STAGE_NAMES: &[&str] = &["stage1", "stage2"];
const NON_CLAIMS: &[&str] = &[
    "compiler-correctness",
    "seed-correctness",
    "kernel-isolation",
    "independent-rebuild-agreement",
    "bit-for-bit-release-reproducibility",
    "deployment-success",
    "full-cargo-compatibility",
];
const PARITY_AXES: &[&str] = &["live-bootstrap", "guix", "stagex"];
const READ_BUFFER_BYTES: usize = 1_048_576;
const MEMBER_BYTES_MAX: u64 = 700_000_000;
const PATH_BYTES_MAX: usize = 1_024;
const ZSTD_LEVEL: i32 = 19;
const BLAKE3_HEX_LENGTH: usize = 64;

#[derive(Debug)]
struct Args {
    project_root: PathBuf,
    proof_root: PathBuf,
    parity_report: PathBuf,
    source_commit: String,
    out: PathBuf,
}

#[derive(Debug, Clone)]
struct SourceSpec {
    role: String,
    source_class: String,
    relative_path: String,
    expected_schema: String,
    path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct BundleManifest {
    schema: String,
    release_id: String,
    source_commit: String,
    source_blake3: String,
    selected_provider_kind: String,
    parity_axes: Vec<String>,
    witness_policy: String,
    witness_sidecar_count: u32,
    members: Vec<BundleMember>,
    non_claims: Vec<String>,
    manifest_identity_blake3: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BundleMember {
    role: String,
    source_class: String,
    source_path: String,
    bundle_path: String,
    expected_schema: String,
    encoding: String,
    content_blake3: String,
    content_bytes: u64,
    compressed_blake3: String,
    compressed_bytes: u64,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("source-built parity bundle export failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1))?;
    validate_args(&args)?;
    let descriptor = read_json(&args.project_root.join(DESCRIPTOR_PATH))?;
    let specs = collect_source_specs(&args, &descriptor)?;
    let stage = stage_path(&args.out)?;
    fs::create_dir_all(stage.join("members")).map_err(|error| format!("create stage: {error}"))?;
    let result = export_members(&stage, &specs).and_then(|members| write_manifest(&stage, &descriptor, &args, members));
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    fs::rename(&stage, &args.out).map_err(|error| format!("publish {}: {error}", args.out.display()))?;
    println!("exported source-built parity bundle: {}", args.out.display());
    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<Args, String>
where I: Iterator<Item = String> {
    let mut values = BTreeMap::new();
    while let Some(option) = args.next() {
        let value = args.next().ok_or_else(|| format!("{option} requires a value"))?;
        if values.insert(option.clone(), value).is_some() {
            return Err(format!("duplicate option: {option}"));
        }
    }
    Ok(Args {
        project_root: required_path(&values, "--project-root")?,
        proof_root: required_path(&values, "--proof-root")?,
        parity_report: required_path(&values, "--parity-report")?,
        source_commit: required_string(&values, "--source-commit")?,
        out: required_path(&values, "--out")?,
    })
}

fn required_path(values: &BTreeMap<String, String>, option: &str) -> Result<PathBuf, String> {
    required_string(values, option).map(PathBuf::from)
}

fn required_string(values: &BTreeMap<String, String>, option: &str) -> Result<String, String> {
    values.get(option).cloned().ok_or_else(|| format!("missing {option}"))
}

fn validate_args(args: &Args) -> Result<(), String> {
    if !args.project_root.is_dir() || !args.proof_root.is_dir() || !args.parity_report.is_file() {
        return Err("project root, proof root, or parity report is unavailable".to_string());
    }
    if args.out.exists() {
        return Err(format!("output already exists: {}", args.out.display()));
    }
    if args.source_commit.len() != 40 || !args.source_commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("source commit must be a 40-character hexadecimal identity".to_string());
    }
    Ok(())
}

fn collect_source_specs(args: &Args, descriptor: &Value) -> Result<Vec<SourceSpec>, String> {
    let mut specs = Vec::new();
    add_project_spec(
        &mut specs,
        args,
        "provider-linkage",
        PROVIDER_LINKAGE_PATH,
        "mantle-self-build-provider-kind-linkage-v1",
    );
    add_project_spec(
        &mut specs,
        args,
        "stagex-lineage-receipt",
        STAGEX_RECEIPT_PATH,
        "mantle-stagex-lineage-provider-receipt-v1",
    );
    for (role, path) in ROW_RECEIPT_PATHS {
        add_project_spec(&mut specs, args, role, path, row_schema(role));
    }
    add_bound_specs(&mut specs, args, descriptor)?;
    add_adapter_specs(&mut specs, args, descriptor)?;
    add_action_detail_specs(&mut specs, args);
    specs.push(SourceSpec {
        role: "parity-report".to_string(),
        source_class: "generated".to_string(),
        relative_path: "generated/parity-report.json".to_string(),
        expected_schema: "crunch-bootstrap-parity-gap-report-v1".to_string(),
        path: args.parity_report.clone(),
    });
    specs.sort_by(|left, right| left.role.cmp(&right.role));
    require_unique_specs(&specs)?;
    Ok(specs)
}

fn add_project_spec(specs: &mut Vec<SourceSpec>, args: &Args, role: &str, path: &str, schema: &str) {
    specs.push(SourceSpec {
        role: role.to_string(),
        source_class: "repository".to_string(),
        relative_path: path.to_string(),
        expected_schema: schema.to_string(),
        path: args.project_root.join(path),
    });
}

fn row_schema(role: &str) -> &'static str {
    if role.starts_with("row-early-") {
        "mantle-early-native-row-receipt-v2"
    } else {
        "mantle-final-native-row-receipt-v1"
    }
}

fn add_bound_specs(specs: &mut Vec<SourceSpec>, args: &Args, descriptor: &Value) -> Result<(), String> {
    let bindings = object_field(descriptor, "bound_evidence")?;
    for (key, role, schema) in [
        ("deterministic_proof", "deterministic-proof", "mantle-deterministic-proof-receipt-v2"),
        ("action_plan", "root-action-plan", "mantle-root-action-trust-plan-v1"),
        ("action_reconciliation", "root-action-reconciliation", "mantle-root-action-reconciliation-v1"),
        ("trust_report", "trust-report", "mantle-bootstrap-trust-report-v1"),
    ] {
        let binding = bindings.get(key).and_then(Value::as_object).ok_or_else(|| format!("missing binding {key}"))?;
        let path = string_field_map(binding, "path")?;
        validate_relative_path(path)?;
        add_project_spec(specs, args, role, path, schema);
    }
    Ok(())
}

fn add_adapter_specs(specs: &mut Vec<SourceSpec>, args: &Args, descriptor: &Value) -> Result<(), String> {
    let bindings = object_field(descriptor, "bound_evidence")?;
    let plan_binding = bindings
        .get("action_plan")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing root action plan binding".to_string())?;
    let plan_path = string_field_map(plan_binding, "path")?;
    let plan = read_json(&args.project_root.join(plan_path))?;
    let adapters = plan
        .get("adapters")
        .and_then(Value::as_array)
        .ok_or_else(|| "root action adapters missing".to_string())?;
    for adapter in adapters {
        let id = string_field(adapter, "adapter_id")?;
        let plan_relative = string_field(adapter, "plan_path")?;
        let reconciliation_relative = string_field(adapter, "reconciliation_path")?;
        specs.push(proof_spec(args, &format!("adapter-{id}-plan"), plan_relative, adapter_plan_schema(id))?);
        specs.push(proof_spec(
            args,
            &format!("adapter-{id}-reconciliation"),
            reconciliation_relative,
            adapter_reconciliation_schema(id),
        )?);
    }
    Ok(())
}

fn adapter_plan_schema(adapter: &str) -> &'static str {
    match adapter {
        "native-provider" => "mantle-eager-derivation-action-plan-v1",
        "rust-provider" => "mantle-source-built-rust-provider-action-plan-v1",
        "rust-units-stage1" | "rust-units-stage2" => "mantle-source-built-rust-child-action-plan-v1",
        "stagex" => "mantle-stagex-materialization-plan-v1",
        _ => "unsupported-adapter-plan",
    }
}

fn adapter_reconciliation_schema(adapter: &str) -> &'static str {
    match adapter {
        "native-provider" => "mantle-eager-derivation-reconciliation-v1",
        "rust-provider" => "mantle-source-built-rust-provider-action-reconciliation-v1",
        "rust-units-stage1" | "rust-units-stage2" => "mantle-source-built-rust-child-action-reconciliation-v1",
        "stagex" => "json-array:protected-exec-audit-v1",
        _ => "unsupported-adapter-reconciliation",
    }
}

fn add_action_detail_specs(specs: &mut Vec<SourceSpec>, args: &Args) {
    for name in ["rust-provider-action-audit.json", "rust-provider-action-authority.json"] {
        let role = name.trim_end_matches(".json");
        specs.push(SourceSpec {
            role: role.to_string(),
            source_class: "proof".to_string(),
            relative_path: format!("{RUST_PROVIDER_ACTION_ROOT}/{name}"),
            expected_schema: format!("mantle-source-built-{role}-v1"),
            path: args.proof_root.join(RUST_PROVIDER_ACTION_ROOT).join(name),
        });
    }
    for stage in RUST_STAGE_NAMES {
        for (suffix, schema_suffix) in [
            ("plan", "stage-action-plan"),
            ("reconciliation", "stage-action-reconciliation"),
        ] {
            let relative = format!("{RUST_PROVIDER_ACTION_ROOT}/stages/{stage}-{suffix}.json");
            specs.push(SourceSpec {
                role: format!("rust-provider-{}-{suffix}", stage.replace('_', "-")),
                source_class: "proof".to_string(),
                relative_path: relative.clone(),
                expected_schema: format!("mantle-source-built-rust-provider-{schema_suffix}-v1"),
                path: args.proof_root.join(relative),
            });
        }
    }
    for stage in STAGE_NAMES {
        for (name, schema) in [
            ("audit", "mantle-source-built-rust-child-action-audit-v1"),
            ("authority", "mantle-source-built-rust-child-action-authority-v1"),
        ] {
            let relative = format!("cargo-free-fixed-point/{stage}/rust-child-actions/{name}.json");
            specs.push(SourceSpec {
                role: format!("rust-units-{stage}-{name}"),
                source_class: "proof".to_string(),
                relative_path: relative.clone(),
                expected_schema: schema.to_string(),
                path: args.proof_root.join(relative),
            });
        }
    }
}

fn proof_spec(args: &Args, role: &str, relative: &str, schema: &str) -> Result<SourceSpec, String> {
    validate_relative_path(relative)?;
    Ok(SourceSpec {
        role: role.to_string(),
        source_class: "proof".to_string(),
        relative_path: relative.to_string(),
        expected_schema: schema.to_string(),
        path: args.proof_root.join(relative),
    })
}

fn require_unique_specs(specs: &[SourceSpec]) -> Result<(), String> {
    let mut roles = BTreeMap::new();
    for spec in specs {
        validate_role(&spec.role)?;
        if roles.insert(spec.role.as_str(), spec.path.as_path()).is_some() {
            return Err(format!("duplicate bundle role: {}", spec.role));
        }
    }
    Ok(())
}

fn export_members(stage: &Path, specs: &[SourceSpec]) -> Result<Vec<BundleMember>, String> {
    let mut members = Vec::with_capacity(specs.len());
    for spec in specs {
        members.push(export_member(stage, spec)?);
    }
    Ok(members)
}

fn export_member(stage: &Path, spec: &SourceSpec) -> Result<BundleMember, String> {
    let metadata =
        fs::symlink_metadata(&spec.path).map_err(|error| format!("metadata {}: {error}", spec.path.display()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || metadata.len() > MEMBER_BYTES_MAX {
        return Err(format!("member is not a bounded regular file: {}", spec.path.display()));
    }
    let bundle_path = format!("members/{}.json.zst", spec.role);
    let output_path = stage.join(&bundle_path);
    let (content_blake3, content_bytes) = compress_member(&spec.path, &output_path)?;
    let compressed_blake3 = blake3_file(&output_path)?;
    let compressed_bytes = fs::metadata(&output_path)
        .map_err(|error| format!("metadata {}: {error}", output_path.display()))?
        .len();
    Ok(BundleMember {
        role: spec.role.clone(),
        source_class: spec.source_class.clone(),
        source_path: spec.relative_path.clone(),
        bundle_path,
        expected_schema: spec.expected_schema.clone(),
        encoding: MEMBER_ENCODING.to_string(),
        content_blake3,
        content_bytes,
        compressed_blake3,
        compressed_bytes,
    })
}

fn compress_member(source: &Path, output: &Path) -> Result<(String, u64), String> {
    let mut input = File::open(source).map_err(|error| format!("open {}: {error}", source.display()))?;
    let output_file = File::create_new(output).map_err(|error| format!("create {}: {error}", output.display()))?;
    let mut encoder = zstd::stream::write::Encoder::new(output_file, ZSTD_LEVEL)
        .map_err(|error| format!("start zstd {}: {error}", output.display()))?;
    encoder.include_checksum(true).map_err(|error| format!("configure zstd: {error}"))?;
    let mut hasher = blake3::Hasher::new();
    let mut bytes = 0_u64;
    let mut buffer = vec![0_u8; READ_BUFFER_BYTES];
    loop {
        let count = input.read(&mut buffer).map_err(|error| format!("read {}: {error}", source.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        encoder
            .write_all(&buffer[..count])
            .map_err(|error| format!("compress {}: {error}", source.display()))?;
        bytes = bytes
            .checked_add(u64::try_from(count).map_err(|_| "read size exceeds u64")?)
            .ok_or("member byte count overflow")?;
        if bytes > MEMBER_BYTES_MAX {
            return Err(format!("member exceeds byte bound: {}", source.display()));
        }
    }
    encoder.finish().map_err(|error| format!("finish zstd {}: {error}", output.display()))?;
    Ok((hasher.finalize().to_hex().to_string(), bytes))
}

fn write_manifest(stage: &Path, descriptor: &Value, args: &Args, members: Vec<BundleMember>) -> Result<(), String> {
    let source_blake3 = string_field(descriptor, "source_blake3")?;
    require_blake3(source_blake3, "source_blake3")?;
    let mut manifest = BundleManifest {
        schema: BUNDLE_SCHEMA.to_string(),
        release_id: RELEASE_ID.to_string(),
        source_commit: args.source_commit.clone(),
        source_blake3: source_blake3.to_string(),
        selected_provider_kind: PROVIDER_KIND.to_string(),
        parity_axes: PARITY_AXES.iter().map(|axis| (*axis).to_string()).collect(),
        witness_policy: "not-selected".to_string(),
        witness_sidecar_count: 0,
        members,
        non_claims: NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect(),
        manifest_identity_blake3: String::new(),
    };
    manifest.manifest_identity_blake3 = manifest_identity(&manifest)?;
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| format!("serialize manifest: {error}"))?;
    fs::write(stage.join("manifest.json"), bytes).map_err(|error| format!("write manifest: {error}"))
}

fn manifest_identity(manifest: &BundleManifest) -> Result<String, String> {
    let mut value = serde_json::to_value(manifest).map_err(|error| format!("encode manifest identity: {error}"))?;
    value["manifest_identity_blake3"] = Value::String(String::new());
    let bytes = serde_json::to_vec(&value).map_err(|error| format!("serialize manifest identity: {error}"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn stage_path(out: &Path) -> Result<PathBuf, String> {
    let file_name = out.file_name().and_then(|name| name.to_str()).ok_or("output needs a UTF-8 file name")?;
    let parent = out.parent().ok_or("output needs a parent")?;
    Ok(parent.join(format!(".{file_name}.tmp-{}", std::process::id())))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn object_field<'a>(value: &'a Value, field: &str) -> Result<&'a serde_json::Map<String, Value>, String> {
    value.get(field).and_then(Value::as_object).ok_or_else(|| format!("field {field} is not an object"))
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("field {field} is not a string"))
}

fn string_field_map<'a>(value: &'a serde_json::Map<String, Value>, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("field {field} is not a string"))
}

fn validate_relative_path(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > PATH_BYTES_MAX {
        return Err(format!("path is empty or too long: {value:?}"));
    }
    let path = Path::new(value);
    if path.is_absolute() || !path.components().all(|component| matches!(component, Component::Normal(_))) {
        return Err(format!("path is not a safe relative path: {value}"));
    }
    Ok(())
}

fn validate_role(value: &str) -> Result<(), String> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(format!("bundle role is unsafe: {value}"));
    }
    Ok(())
}

fn require_blake3(value: &str, field: &str) -> Result<(), String> {
    if value.len() == BLAKE3_HEX_LENGTH
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(format!("{field} is not lowercase BLAKE3"))
    }
}

fn blake3_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; READ_BUFFER_BYTES];
    loop {
        let count = file.read(&mut buffer).map_err(|error| format!("read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}
