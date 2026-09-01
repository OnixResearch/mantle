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
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

const BUNDLE_SCHEMA: &str = "mantle-full-bootstrap-parity-bundle-v1";
const VERIFICATION_SCHEMA: &str = "mantle-full-bootstrap-parity-verification-v1";
const MEMBER_ENCODING: &str = "zstd-json-v1";
const REQUIRED_PROVIDER_KIND: &str = "full-source";
const REQUIRED_PROOF_SCHEMA: &str = "mantle-deterministic-proof-receipt-v2";
const REQUIRED_VERDICT: &str = "self-rebuild-match";
const REQUIRED_HERMETICITY: &str = "strict";
const REQUIRED_TRUST_STATUS: &str = "complete";
const DEFAULT_BUNDLE: &str = "bootstrap/evidence/full-bootstrap-parity-v98";
const MANIFEST_FILE: &str = "manifest.json";
const EXPECTED_AXES: &[&str] = &["live-bootstrap", "guix", "stagex"];
const EXPECTED_ROWS: &[(&str, &str)] = &[
    ("row-early-binutils", "binutils.tcc"),
    ("row-early-gcc40", "gcc.4.0"),
    ("row-final-gcc47", "gcc.4.7"),
    ("row-final-gcc10", "gcc.10"),
    ("row-final-musl-binutils", "full-musl-binutils"),
];
const EXPECTED_ADAPTERS: &[&str] = &[
    "native-provider",
    "rust-provider",
    "rust-units-stage1",
    "rust-units-stage2",
    "stagex",
];
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "compiler-correctness",
    "seed-correctness",
    "kernel-isolation",
    "independent-rebuild-agreement",
    "bit-for-bit-release-reproducibility",
    "deployment-success",
    "full-cargo-compatibility",
];
const REQUIRED_ROLES: &[&str] = &[
    "deterministic-proof",
    "provider-linkage",
    "root-action-plan",
    "root-action-reconciliation",
    "stagex-lineage-receipt",
    "trust-report",
    "parity-report",
    "adapter-native-provider-plan",
    "adapter-native-provider-reconciliation",
    "adapter-rust-provider-plan",
    "adapter-rust-provider-reconciliation",
    "adapter-rust-units-stage1-plan",
    "adapter-rust-units-stage1-reconciliation",
    "adapter-rust-units-stage2-plan",
    "adapter-rust-units-stage2-reconciliation",
    "adapter-stagex-plan",
    "adapter-stagex-reconciliation",
    "rust-provider-action-audit",
    "rust-provider-action-authority",
    "rust-units-stage1-audit",
    "rust-units-stage1-authority",
    "rust-units-stage2-audit",
    "rust-units-stage2-authority",
];
const READ_BUFFER_BYTES: usize = 1_048_576;
const SMALL_JSON_BYTES_MAX: u64 = 32 * 1_048_576;
const MEMBER_BYTES_MAX: u64 = 700_000_000;
const SOURCE_BYTES_MAX: u64 = 32 * 1_048_576;
const MEMBER_COUNT_MAX: usize = 128;
const ACTION_COUNT_MAX: u64 = 4_096;
const EVENT_COUNT_MAX: u64 = 1_000_000;
const PATH_BYTES_MAX: usize = 1_024;
const BLAKE3_HEX_LENGTH: usize = 64;
const SOURCE_COMMIT_HEX_LENGTH: usize = 40;
const REQUIRED_STAGE_COUNT: u64 = 6;
const REQUIRED_EXECUTED_STAGE_COUNT: u64 = 2;
const REQUIRED_RESTORED_STAGE_COUNT: u64 = 4;
const REQUIRED_RUN_COUNT: usize = 2;

#[derive(Debug)]
struct Args {
    root: PathBuf,
    bundle: PathBuf,
    output: Option<PathBuf>,
    self_test: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Debug)]
struct LoadedBundle {
    manifest: BundleManifest,
    values: BTreeMap<String, Value>,
    members: BTreeMap<String, BundleMember>,
    bundle_dir: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
struct AuditEvent {
    digest_hex: String,
    phase: String,
    inventory_entry_id: Option<String>,
    policy_decision: String,
}

#[derive(Debug, Deserialize)]
struct ActionAudit {
    schema: String,
    action_plan_digest_blake3: String,
    raw_event_count: u64,
    #[serde(default)]
    assigned_event_count: Option<u64>,
    #[serde(default)]
    promotion_count: Option<u64>,
    raw_events: Vec<AuditEvent>,
}

#[derive(Debug, Serialize)]
struct VerificationReceipt {
    schema: String,
    status: String,
    bundle_manifest_identity_blake3: String,
    source_commit: String,
    source_blake3: String,
    selected_provider_kind: String,
    parity_axes: Vec<String>,
    native_row_count: u64,
    adapter_count: u64,
    planned_actions: u64,
    matched_actions: u64,
    observed_events: u64,
    matched_events: u64,
    local_only: bool,
    witness_policy: String,
    witness_sidecar_count: u32,
    blockers: Vec<String>,
    non_claims: Vec<String>,
    verification_identity_blake3: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("source-built parity promotion check failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1))?;
    if args.self_test {
        return self_test();
    }
    let bundle = load_bundle(&args.root, &args.bundle)?;
    let mut receipt = validate_bundle(&args.root, &bundle)?;
    receipt.verification_identity_blake3 = verification_identity(&receipt)?;
    let bytes = serde_json::to_vec_pretty(&receipt).map_err(|error| format!("serialize receipt: {error}"))?;
    if let Some(output) = args.output {
        write_new_file(&output, &bytes)?;
    } else {
        println!("{}", String::from_utf8(bytes).map_err(|error| format!("receipt UTF-8: {error}"))?);
    }
    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<Args, String>
where I: Iterator<Item = String> {
    let mut root = PathBuf::from(".");
    let mut bundle = PathBuf::from(DEFAULT_BUNDLE);
    let mut output = None;
    let mut self_test = false;
    while let Some(option) = args.next() {
        match option.as_str() {
            "--root" => root = next_path(&mut args, "--root")?,
            "--bundle" => bundle = next_path(&mut args, "--bundle")?,
            "--out" => output = Some(next_path(&mut args, "--out")?),
            "--self-test" => self_test = true,
            "-h" | "--help" => return Err(usage()),
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }
    Ok(Args {
        root,
        bundle,
        output,
        self_test,
    })
}

fn next_path<I>(args: &mut I, option: &str) -> Result<PathBuf, String>
where I: Iterator<Item = String> {
    args.next().map(PathBuf::from).ok_or_else(|| format!("{option} requires a path"))
}

fn usage() -> String {
    "usage: cargo -Zscript scripts/check-source-built-parity-promotion.rs [--root ROOT] [--bundle DIR] [--out FILE] [--self-test]".to_string()
}

fn load_bundle(root: &Path, bundle: &Path) -> Result<LoadedBundle, String> {
    if !root.is_dir() {
        return Err(format!("root is not a directory: {}", root.display()));
    }
    let bundle_dir = if bundle.is_absolute() {
        bundle.to_path_buf()
    } else {
        root.join(bundle)
    };
    let manifest_path = bundle_dir.join(MANIFEST_FILE);
    let manifest_bytes = read_bounded_regular(&manifest_path, SOURCE_BYTES_MAX)?;
    let manifest: BundleManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let mut values = BTreeMap::new();
    let mut members = BTreeMap::new();
    for member in &manifest.members {
        let value = validate_member(&bundle_dir, member)?;
        if let Some(value) = value {
            values.insert(member.role.clone(), value);
        }
        members.insert(member.role.clone(), member.clone());
    }
    Ok(LoadedBundle {
        manifest,
        values,
        members,
        bundle_dir,
    })
}

fn validate_manifest(manifest: &BundleManifest) -> Result<(), String> {
    if manifest.schema != BUNDLE_SCHEMA || manifest.selected_provider_kind != REQUIRED_PROVIDER_KIND {
        return Err("bundle schema or provider kind is invalid".to_string());
    }
    require_blake3(&manifest.source_blake3, "manifest source")?;
    require_hex(&manifest.source_commit, SOURCE_COMMIT_HEX_LENGTH, "source commit")?;
    if manifest.members.is_empty() || manifest.members.len() > MEMBER_COUNT_MAX {
        return Err("bundle member count is outside its bound".to_string());
    }
    require_exact_strings(&manifest.parity_axes, EXPECTED_AXES, "parity axes")?;
    require_exact_strings(&manifest.non_claims, REQUIRED_NON_CLAIMS, "non-claims")?;
    validate_witness_independence(&manifest.witness_policy, manifest.witness_sidecar_count)?;
    let mut roles = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for member in &manifest.members {
        validate_role(&member.role)?;
        validate_relative_path(&member.source_path)?;
        validate_relative_path(&member.bundle_path)?;
        require_blake3(&member.content_blake3, "member content")?;
        require_blake3(&member.compressed_blake3, "member compressed")?;
        if member.encoding != MEMBER_ENCODING || member.content_bytes == 0 || member.content_bytes > MEMBER_BYTES_MAX {
            return Err(format!("member {} encoding or size is invalid", member.role));
        }
        if !roles.insert(member.role.as_str()) || !paths.insert(member.bundle_path.as_str()) {
            return Err("bundle member role or path is duplicated".to_string());
        }
    }
    for required in REQUIRED_ROLES {
        if !roles.contains(required) {
            return Err(format!("required bundle role is missing: {required}"));
        }
    }
    let expected_identity = manifest_identity(manifest)?;
    require_equal("manifest identity", &manifest.manifest_identity_blake3, &expected_identity)
}

fn validate_member(bundle_dir: &Path, member: &BundleMember) -> Result<Option<Value>, String> {
    let path = bundle_dir.join(&member.bundle_path);
    let metadata = fs::symlink_metadata(&path).map_err(|error| format!("metadata {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(format!("bundle member is not a regular no-follow file: {}", path.display()));
    }
    if metadata.len() != member.compressed_bytes {
        return Err(format!("bundle member compressed size drift: {}", member.role));
    }
    let compressed_digest = blake3_file(&path)?;
    require_equal("compressed member digest", &compressed_digest, &member.compressed_blake3)?;
    let collect = member.content_bytes <= SMALL_JSON_BYTES_MAX;
    let (content_digest, content_bytes, bytes) = decode_member(&path, collect)?;
    require_equal("member content digest", &content_digest, &member.content_blake3)?;
    if content_bytes != member.content_bytes {
        return Err(format!("member {} uncompressed size drift", member.role));
    }
    if let Some(bytes) = bytes {
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|error| format!("parse member {}: {error}", member.role))?;
        validate_expected_schema(&value, &member.expected_schema, &member.role)?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn decode_member(path: &Path, collect: bool) -> Result<(String, u64, Option<Vec<u8>>), String> {
    let file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut decoder =
        zstd::stream::read::Decoder::new(file).map_err(|error| format!("decode {}: {error}", path.display()))?;
    let mut hasher = blake3::Hasher::new();
    let mut total = 0_u64;
    let mut collected = collect.then(Vec::new);
    let mut buffer = vec![0_u8; READ_BUFFER_BYTES];
    loop {
        let count = decoder.read(&mut buffer).map_err(|error| format!("decode {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(u64::try_from(count).map_err(|_| "decoded size exceeds u64")?)
            .ok_or("decoded size overflow")?;
        if total > MEMBER_BYTES_MAX {
            return Err(format!("decoded member exceeds byte bound: {}", path.display()));
        }
        hasher.update(&buffer[..count]);
        if let Some(bytes) = &mut collected {
            bytes.extend_from_slice(&buffer[..count]);
        }
    }
    Ok((hasher.finalize().to_hex().to_string(), total, collected))
}

fn validate_bundle(root: &Path, bundle: &LoadedBundle) -> Result<VerificationReceipt, String> {
    validate_descriptor(bundle)?;
    validate_native_rows(root, bundle)?;
    validate_stagex(bundle)?;
    let action = validate_action_domains(bundle)?;
    validate_parity_report(bundle)?;
    Ok(VerificationReceipt {
        schema: VERIFICATION_SCHEMA.to_string(),
        status: "complete".to_string(),
        bundle_manifest_identity_blake3: bundle.manifest.manifest_identity_blake3.clone(),
        source_commit: bundle.manifest.source_commit.clone(),
        source_blake3: bundle.manifest.source_blake3.clone(),
        selected_provider_kind: REQUIRED_PROVIDER_KIND.to_string(),
        parity_axes: bundle.manifest.parity_axes.clone(),
        native_row_count: u64::try_from(EXPECTED_ROWS.len()).map_err(|_| "row count exceeds u64")?,
        adapter_count: u64::try_from(EXPECTED_ADAPTERS.len()).map_err(|_| "adapter count exceeds u64")?,
        planned_actions: action.0,
        matched_actions: action.1,
        observed_events: action.2,
        matched_events: action.3,
        local_only: true,
        witness_policy: bundle.manifest.witness_policy.clone(),
        witness_sidecar_count: bundle.manifest.witness_sidecar_count,
        blockers: Vec::new(),
        non_claims: bundle.manifest.non_claims.clone(),
        verification_identity_blake3: String::new(),
    })
}

fn validate_descriptor(bundle: &LoadedBundle) -> Result<(), String> {
    let deterministic = value(bundle, "deterministic-proof")?;
    require_schema(deterministic, REQUIRED_PROOF_SCHEMA)?;
    require_string(deterministic, "workflow_version", REQUIRED_PROOF_SCHEMA)?;
    require_string(deterministic, "verdict", REQUIRED_VERDICT)?;
    require_string(deterministic, "selected_provider_kind", REQUIRED_PROVIDER_KIND)?;
    require_string(deterministic, "hermeticity_mode", REQUIRED_HERMETICITY)?;
    require_empty_array(deterministic, "blocking_reasons")?;
    validate_proof_runs(deterministic)?;
    let linkage = value(bundle, "provider-linkage")?;
    require_schema(linkage, "mantle-self-build-provider-kind-linkage-v1")?;
    for object in ["proof_identity", "proof_linkage", "prerequisites"] {
        require_string(
            object_field(linkage, object)?,
            if object == "prerequisites" {
                "provider_kind"
            } else {
                "selected_provider_kind"
            },
            REQUIRED_PROVIDER_KIND,
        )?;
    }
    validate_trust_report(bundle, deterministic)
}

fn validate_proof_runs(proof: &Value) -> Result<(), String> {
    let runs = array_field(proof, "runs")?;
    if runs.len() != REQUIRED_RUN_COUNT {
        return Err("deterministic proof run count is invalid".to_string());
    }
    let mut output = None;
    for run in runs {
        require_empty_array(run, "authority_violations")?;
        require_empty_array(run, "hermeticity_audit_events")?;
        require_empty_array(run, "substituted_dependency_identities")?;
        let digest = string_field(
            array_field(run, "output_digests")?.first().ok_or("proof run has no output")?,
            "digest_blake3",
        )?;
        require_blake3(digest, "proof output")?;
        if let Some(expected) = output {
            require_equal("proof output", digest, expected)?;
        } else {
            output = Some(digest);
        }
    }
    Ok(())
}

fn validate_trust_report(bundle: &LoadedBundle, proof: &Value) -> Result<(), String> {
    let trust = value(bundle, "trust-report")?;
    require_schema(trust, "mantle-bootstrap-trust-report-v1")?;
    require_string(trust, "status", REQUIRED_TRUST_STATUS)?;
    require_bool(trust, "fixed_point_verified", true)?;
    require_bool(trust, "root_action_trust_complete", true)?;
    require_empty_array(trust, "blockers")?;
    let proof_summary = object_field(trust, "proof")?;
    require_string(proof_summary, "receipt_digest_blake3", string_field(proof, "receipt_blake3")?)?;
    require_string(proof_summary, "source_authority_digest_blake3", &bundle.manifest.source_blake3)?;
    require_string(proof_summary, "provider_kind", REQUIRED_PROVIDER_KIND)?;
    require_string(proof_summary, "hermeticity_mode", REQUIRED_HERMETICITY)?;
    require_zero(proof_summary, "substitution_count")?;
    let stages = object_field(trust, "stages")?;
    require_count(stages, "planned", REQUIRED_STAGE_COUNT)?;
    require_count(stages, "observed", REQUIRED_STAGE_COUNT)?;
    require_count(stages, "executed", REQUIRED_EXECUTED_STAGE_COUNT)?;
    require_count(stages, "restored_checkpoint", REQUIRED_RESTORED_STAGE_COUNT)?;
    require_zero(stages, "authority_violations")?;
    require_zero(stages, "fallback_events")
}

fn validate_native_rows(root: &Path, bundle: &LoadedBundle) -> Result<(), String> {
    for (role, row_id) in EXPECTED_ROWS {
        let row = value(bundle, role)?;
        require_string(row, "row_id", row_id)?;
        validate_row_sources(root, row)?;
        validate_row_generated(row)?;
        validate_row_behavior(row)?;
        validate_trust_and_fallback(row)?;
        validate_row_artifact_links(root, row)?;
    }
    Ok(())
}

fn validate_row_sources(root: &Path, row: &Value) -> Result<(), String> {
    let records = array_field(row, "source_records")?;
    if records.is_empty() {
        return Err("native row source record list is empty".to_string());
    }
    for record in records {
        let relative = string_field(record, "path")?;
        let expected = string_field(record, "blake3")?;
        validate_relative_path(relative)?;
        require_blake3(expected, "row source")?;
        let bytes = read_bounded_regular(&root.join(relative), SOURCE_BYTES_MAX)?;
        require_equal("row source digest", blake3::hash(&bytes).to_hex().as_str(), expected)?;
    }
    Ok(())
}

fn validate_row_generated(row: &Value) -> Result<(), String> {
    for artifact in array_field(row, "generated_artifacts")? {
        require_non_empty(artifact, "id")?;
        require_string(artifact, "status", "regenerated")?;
    }
    Ok(())
}

fn validate_row_behavior(row: &Value) -> Result<(), String> {
    let behavior = object_field(row, "behavior")?;
    for field in ["positive", "rejection"] {
        let cases = array_field(behavior, field)?;
        if cases.is_empty() {
            return Err(format!("row {field} matrix is empty"));
        }
        for case in cases {
            require_non_empty(case, "id")?;
            require_string(case, "status", "passed")?;
        }
    }
    Ok(())
}

fn validate_trust_and_fallback(row: &Value) -> Result<(), String> {
    let trust = object_field(row, "trust")?;
    require_bool(trust, "trust_unsigned", false)?;
    require_bool(trust, "signing_key_selected", true)?;
    require_bool(trust, "substitutions_enabled", false)?;
    let fallback = object_field(row, "fallback")?;
    for field in [
        "host_tool_execution",
        "predecessor_delegation",
        "release_generated_substitution",
        "configure_bridge_compiler_use",
    ] {
        require_bool(fallback, field, false)?;
    }
    for field in ["fabricated_objects", "omitted_artifacts"] {
        require_empty_array(fallback, field)?;
    }
    Ok(())
}

fn validate_row_artifact_links(root: &Path, row: &Value) -> Result<(), String> {
    for predecessor in array_field(row, "predecessors")? {
        validate_artifact_reference(root, predecessor, "artifact_evidence_path", "artifact_attestation_blake3")?;
    }
    let output = object_field(row, "output")?;
    validate_artifact_reference(root, output, "artifact_evidence_path", "artifact_attestation_blake3")?;
    let acceptance_path = string_field(row, "acceptance_evidence_path")?;
    let acceptance = read_json_reference(root, acceptance_path)?;
    require_string(&acceptance, "row_id", string_field(row, "row_id")?)?;
    require_string(&acceptance, "logical_path", string_field(output, "logical_path")?)?;
    validate_trust_and_fallback(&acceptance)
}

fn validate_artifact_reference(root: &Path, value: &Value, path_field: &str, digest_field: &str) -> Result<(), String> {
    let path = string_field(value, path_field)?;
    let expected = string_field(value, digest_field)?;
    let artifact = read_json_reference(root, path)?;
    require_string(&artifact, "digest", expected)?;
    require_string(
        object_field(&artifact, "attestation")?.get("facts").ok_or("artifact facts missing")?,
        "logical_path",
        string_field(value, "logical_path")?,
    )
}

fn read_json_reference(root: &Path, relative: &str) -> Result<Value, String> {
    validate_relative_path(relative)?;
    let bytes = read_bounded_regular(&root.join(relative), SOURCE_BYTES_MAX)?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse reference {relative}: {error}"))
}

fn validate_stagex(bundle: &LoadedBundle) -> Result<(), String> {
    let receipt = value(bundle, "stagex-lineage-receipt")?;
    require_schema(receipt, "mantle-stagex-lineage-provider-receipt-v1")?;
    require_string(receipt, "lineage_receipt_status", "complete")?;
    require_string(receipt, "provider_kind", "stagex-lineage")?;
    require_empty_array(receipt, "fallback_events")?;
    let plan = value(bundle, "adapter-stagex-plan")?;
    require_schema(plan, "mantle-stagex-materialization-plan-v1")?;
    let stages = array_field(plan, "stages")?;
    if u64::try_from(stages.len()).map_err(|_| "StageX stage count exceeds u64")?
        != number_field(plan, "stage_count_max")?
    {
        return Err("StageX plan stage count is inconsistent".to_string());
    }
    let reports = array_field(receipt, "stage_reports")?;
    if reports.len() != stages.len() {
        return Err("StageX report and plan stage counts differ".to_string());
    }
    let plan_ids = stages.iter().map(|stage| string_field(stage, "id")).collect::<Result<BTreeSet<_>, _>>()?;
    let report_ids =
        reports.iter().map(|report| string_field(report, "stage_id")).collect::<Result<BTreeSet<_>, _>>()?;
    if plan_ids != report_ids || plan_ids.len() != stages.len() {
        return Err("StageX stage identity coverage is incomplete".to_string());
    }
    let events: Vec<AuditEvent> = read_large_member(bundle, "adapter-stagex-reconciliation")?;
    validate_audit_events(&events, None)?;
    let observed = u64::try_from(events.len()).map_err(|_| "StageX event count exceeds u64")?;
    if observed != adapter_count(bundle, "stagex", "observed_event_count")? {
        return Err("StageX observed event count differs from the root adapter".to_string());
    }
    Ok(())
}

fn validate_action_domains(bundle: &LoadedBundle) -> Result<(u64, u64, u64, u64), String> {
    let root_plan = value(bundle, "root-action-plan")?;
    let root_reconciliation = value(bundle, "root-action-reconciliation")?;
    require_schema(root_plan, "mantle-root-action-trust-plan-v1")?;
    require_schema(root_reconciliation, "mantle-root-action-reconciliation-v1")?;
    require_bool(root_plan, "local_only", true)?;
    require_bool(root_plan, "cache_only_completion_allowed", false)?;
    require_empty_array(root_plan, "blockers")?;
    let adapters = array_field(root_plan, "adapters")?;
    if adapters.len() != EXPECTED_ADAPTERS.len() {
        return Err("root action adapter count is invalid".to_string());
    }
    for adapter in adapters {
        validate_adapter_binding(bundle, adapter)?;
    }
    validate_native_action_adapter(bundle)?;
    validate_rust_provider_adapter(bundle)?;
    validate_rust_unit_adapter(bundle, "stage1")?;
    validate_rust_unit_adapter(bundle, "stage2")?;
    let planned = number_field(root_reconciliation, "planned_action_count")?;
    let matched = number_field(root_reconciliation, "matched_action_count")?;
    let observed = number_field(root_reconciliation, "observed_event_count")?;
    let matched_events = number_field(root_reconciliation, "matched_event_count")?;
    if planned != matched || observed != matched_events || planned != number_field(root_plan, "action_count")? {
        return Err("root action counts do not reconcile".to_string());
    }
    require_zero_findings(root_reconciliation)?;
    require_bool(root_reconciliation, "local_only", true)?;
    require_empty_array(root_reconciliation, "blockers")?;
    Ok((planned, matched, observed, matched_events))
}

fn validate_adapter_binding(bundle: &LoadedBundle, adapter: &Value) -> Result<(), String> {
    let id = string_field(adapter, "adapter_id")?;
    if !EXPECTED_ADAPTERS.contains(&id) {
        return Err(format!("unknown root adapter: {id}"));
    }
    let plan_role = format!("adapter-{id}-plan");
    let reconciliation_role = format!("adapter-{id}-reconciliation");
    require_equal(
        "adapter plan file digest",
        string_field(adapter, "plan_file_digest_blake3")?,
        &member(bundle, &plan_role)?.content_blake3,
    )?;
    require_equal(
        "adapter reconciliation file digest",
        string_field(adapter, "reconciliation_file_digest_blake3")?,
        &member(bundle, &reconciliation_role)?.content_blake3,
    )?;
    let plan = value(bundle, &plan_role)?;
    if id != "stagex" {
        require_equal(
            "adapter plan semantic digest",
            string_field(adapter, "plan_semantic_digest_blake3")?,
            string_field(plan, "plan_digest_blake3")?,
        )?;
        let reconciliation = value(bundle, &reconciliation_role)?;
        require_equal(
            "adapter reconciliation semantic digest",
            string_field(adapter, "reconciliation_semantic_digest_blake3")?,
            string_field(reconciliation, "reconciliation_digest_blake3")?,
        )?;
    }
    for field in [
        "planned_action_count",
        "matched_action_count",
        "observed_event_count",
        "matched_event_count",
    ] {
        if number_field(adapter, field)? == 0 {
            return Err(format!("adapter {id} has zero {field}"));
        }
    }
    require_bool(adapter, "local_only", true)
}

fn validate_native_action_adapter(bundle: &LoadedBundle) -> Result<(), String> {
    let plan = value(bundle, "adapter-native-provider-plan")?;
    let reconciliation = value(bundle, "adapter-native-provider-reconciliation")?;
    let actions = array_field(plan, "actions")?;
    validate_action_count(plan, actions)?;
    let ids = actions.iter().map(|action| string_field(action, "action_id")).collect::<Result<BTreeSet<_>, _>>()?;
    if ids.len() != actions.len() {
        return Err("native action IDs are duplicated".to_string());
    }
    for action in actions {
        require_bool(action, "local_only", true)?;
        require_blake3(string_field(action, "action_id_blake3")?, "native action")?;
        if array_field(action, "outputs")?.is_empty() {
            return Err("native action has no output authority".to_string());
        }
        let executable = object_field(action, "executable")?;
        let kind = string_field(executable, "kind")?;
        if kind == "fixed-sandbox-shell" {
            require_blake3(string_field(executable, "digest_blake3")?, "native executable")?;
        } else if kind != "builtin" {
            return Err("native action executable authority is unsupported".to_string());
        }
    }
    validate_reconciliation_counts(plan, reconciliation)?;
    require_empty_array(reconciliation, "unknown_event_ids_blake3")?;
    require_empty_array(reconciliation, "missing_action_ids_blake3")?;
    require_empty_array(reconciliation, "overbound_event_ids_blake3")?;
    Ok(())
}

fn validate_rust_provider_adapter(bundle: &LoadedBundle) -> Result<(), String> {
    let plan = value(bundle, "adapter-rust-provider-plan")?;
    let reconciliation = value(bundle, "adapter-rust-provider-reconciliation")?;
    require_bool(plan, "local_only", true)?;
    require_bool(plan, "cache_only_completion_allowed", false)?;
    require_empty_array(plan, "blockers")?;
    let stage_digests = array_field(plan, "stage_plan_digests_blake3")?;
    if stage_digests.len()
        != usize::try_from(number_field(plan, "action_count")?)
            .map_err(|_| "Rust provider action count exceeds usize")?
    {
        return Err("Rust provider stage plan count is inconsistent".to_string());
    }
    let stage_plan_roles = bundle
        .members
        .keys()
        .filter(|role| role.starts_with("rust-provider-") && role.ends_with("-plan"))
        .filter(|role| role.as_str() != "rust-provider-action-plan")
        .cloned()
        .collect::<Vec<_>>();
    if stage_plan_roles.len() != stage_digests.len() {
        return Err("Rust provider exported stage plan count is incomplete".to_string());
    }
    let expected = stage_digests
        .iter()
        .map(|value| value.as_str().ok_or("stage plan digest is not a string"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let observed = stage_plan_roles
        .iter()
        .map(|role| value(bundle, role).and_then(|plan| string_field(plan, "plan_digest_blake3")))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if expected != observed {
        return Err("Rust provider stage plan digest set drifted".to_string());
    }
    let authority = value(bundle, "rust-provider-action-authority")?;
    validate_fixed_authority(authority)?;
    let audit: ActionAudit = read_large_member(bundle, "rust-provider-action-audit")?;
    validate_action_audit(&audit, string_field(plan, "plan_digest_blake3")?)?;
    validate_reconciliation_counts(plan, reconciliation)?;
    require_zero_findings(reconciliation)
}

fn validate_rust_unit_adapter(bundle: &LoadedBundle, stage: &str) -> Result<(), String> {
    let plan_role = format!("adapter-rust-units-{stage}-plan");
    let reconciliation_role = format!("adapter-rust-units-{stage}-reconciliation");
    let audit_role = format!("rust-units-{stage}-audit");
    let authority_role = format!("rust-units-{stage}-authority");
    let plan = value(bundle, &plan_role)?;
    let reconciliation = value(bundle, &reconciliation_role)?;
    let actions = array_field(plan, "actions")?;
    validate_action_count(plan, actions)?;
    let action_ids =
        actions.iter().map(|action| string_field(action, "action_id")).collect::<Result<BTreeSet<_>, _>>()?;
    let fixed = array_field(plan, "fixed_executables")?;
    let fixed_ids =
        fixed.iter().map(|entry| string_field(entry, "authority_id")).collect::<Result<BTreeSet<_>, _>>()?;
    if action_ids.len() != actions.len() || fixed_ids.len() != fixed.len() {
        return Err(format!("Rust {stage} action or authority IDs are duplicated"));
    }
    for entry in fixed {
        validate_fixed_entry(entry)?;
    }
    for action in actions {
        validate_rust_action(action, &action_ids, &fixed_ids)?;
    }
    let authority = value(bundle, &authority_role)?;
    validate_fixed_authority(authority)?;
    let audit = value(bundle, &audit_role)?;
    validate_small_action_audit(audit, string_field(plan, "plan_digest_blake3")?)?;
    validate_reconciliation_counts(plan, reconciliation)?;
    for field in [
        "unknown_event_ids_blake3",
        "denied_event_ids_blake3",
        "drifted_event_ids_blake3",
        "missing_action_ids_blake3",
        "overbound_action_ids_blake3",
    ] {
        require_empty_array(reconciliation, field)?;
    }
    Ok(())
}

fn validate_action_count(plan: &Value, actions: &[Value]) -> Result<(), String> {
    let count = number_field(plan, "action_count")?;
    if count == 0 || count > ACTION_COUNT_MAX || usize::try_from(count).ok() != Some(actions.len()) {
        return Err("action plan count is outside its bound or inconsistent".to_string());
    }
    Ok(())
}

fn validate_fixed_authority(authority: &Value) -> Result<(), String> {
    let fixed = array_field(authority, "fixed_executables")?;
    if fixed.is_empty() {
        return Err("fixed executable authority is empty".to_string());
    }
    let producer_set = authority
        .get("producer_action_ids")
        .and_then(Value::as_array)
        .map(|producers| {
            producers
                .iter()
                .map(|value| value.as_str().ok_or("producer ID is not a string"))
                .collect::<Result<BTreeSet<_>, _>>()
        })
        .transpose()?;
    for entry in fixed {
        validate_fixed_entry(entry)?;
        if producer_set
            .as_ref()
            .is_some_and(|producers| !producers.contains(string_field(entry, "producer_action_id").unwrap_or("")))
        {
            return Err("fixed executable has no producer authority".to_string());
        }
    }
    Ok(())
}

fn validate_fixed_entry(entry: &Value) -> Result<(), String> {
    require_non_empty(entry, "authority_id")?;
    require_non_empty(entry, "producer_action_id")?;
    require_blake3(string_field(entry, "digest_blake3")?, "fixed executable")?;
    let path = string_field(entry, "path")?;
    if !Path::new(path).is_absolute() {
        return Err("fixed executable observation path is relative".to_string());
    }
    Ok(())
}

fn validate_rust_action(action: &Value, action_ids: &BTreeSet<&str>, fixed_ids: &BTreeSet<&str>) -> Result<(), String> {
    let action_id = string_field(action, "action_id")?;
    require_blake3(string_field(action, "action_id_blake3")?, "Rust action")?;
    require_bool(action, "local_only", true)?;
    if array_field(action, "output_identities_blake3")?.is_empty() {
        return Err(format!("Rust action {action_id} has no output identity"));
    }
    for producer in array_field(action, "producer_action_ids")? {
        let producer = producer.as_str().ok_or("Rust producer is not a string")?;
        if !action_ids.contains(producer) {
            return Err("Rust action references an unknown producer".to_string());
        }
    }
    let executable = object_field(action, "executable")?;
    match string_field(executable, "kind")? {
        "fixed" => {
            let authority = string_field(executable, "authority_id")?;
            if !fixed_ids.contains(authority) {
                return Err("Rust action has path-only or unknown fixed authority".to_string());
            }
        }
        "produced" => {
            let producer = string_field(executable, "producer_action_id")?;
            if !action_ids.contains(producer) {
                return Err("Rust produced executable has no producer".to_string());
            }
            require_blake3(string_field(executable, "output_identity_blake3")?, "produced executable")?;
        }
        _ => return Err("Rust action executable kind is unsupported".to_string()),
    }
    Ok(())
}

fn validate_small_action_audit(audit: &Value, expected_plan: &str) -> Result<(), String> {
    require_string(audit, "action_plan_digest_blake3", expected_plan)?;
    let raw_count = number_field(audit, "raw_event_count")?;
    require_count(audit, "assigned_event_count", raw_count)?;
    let events = array_field(audit, "raw_events")?;
    if usize::try_from(raw_count).ok() != Some(events.len()) {
        return Err("Rust unit raw event count is inconsistent".to_string());
    }
    let typed = events
        .iter()
        .map(|event| {
            serde_json::from_value::<AuditEvent>(event.clone()).map_err(|error| format!("decode audit event: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_audit_events(&typed, Some(raw_count))
}

fn validate_action_audit(audit: &ActionAudit, expected_plan: &str) -> Result<(), String> {
    if audit.schema != "mantle-source-built-rust-provider-action-audit-v1" {
        return Err("Rust provider audit schema is invalid".to_string());
    }
    require_equal("Rust provider audit plan", &audit.action_plan_digest_blake3, expected_plan)?;
    if audit.assigned_event_count.is_some_and(|assigned| assigned != audit.raw_event_count) {
        return Err("Rust provider assigned event count differs from raw events".to_string());
    }
    if audit.promotion_count.is_none() {
        return Err("Rust provider audit has no promotion count".to_string());
    }
    validate_audit_events(&audit.raw_events, Some(audit.raw_event_count))
}

fn validate_audit_events(events: &[AuditEvent], expected_count: Option<u64>) -> Result<(), String> {
    let count = u64::try_from(events.len()).map_err(|_| "audit event count exceeds u64")?;
    if count == 0 || count > EVENT_COUNT_MAX || expected_count.is_some_and(|expected| expected != count) {
        return Err("audit event count is outside its bound or inconsistent".to_string());
    }
    for event in events {
        require_blake3(&event.digest_hex, "audit executable")?;
        if event.phase != "protected" || event.policy_decision != "allowed" {
            return Err("audit contains denied, fallback, or unprotected execution".to_string());
        }
        if event.inventory_entry_id.as_deref().is_none_or(str::is_empty) {
            return Err("audit event lacks producer-linked executable authority".to_string());
        }
    }
    Ok(())
}

fn validate_reconciliation_counts(plan: &Value, reconciliation: &Value) -> Result<(), String> {
    let plan_count = number_field(plan, "action_count")?;
    require_count(reconciliation, "planned_action_count", plan_count)?;
    require_count(reconciliation, "matched_action_count", plan_count)?;
    let observed = number_field(reconciliation, "observed_event_count")?;
    require_count(reconciliation, "matched_event_count", observed)?;
    require_bool(reconciliation, "local_only", true)?;
    require_empty_array(reconciliation, "blockers")
}

fn require_zero_findings(value: &Value) -> Result<(), String> {
    for field in [
        "unknown_event_count",
        "missing_action_count",
        "authority_violation_count",
        "fallback_event_count",
        "remote_event_count",
        "cache_only_completion_count",
    ] {
        require_zero(value, field)?;
    }
    Ok(())
}

fn validate_parity_report(bundle: &LoadedBundle) -> Result<(), String> {
    let report = value(bundle, "parity-report")?;
    require_schema(report, "crunch-bootstrap-parity-gap-report-v1")?;
    let axes = array_field(report, "axes")?;
    if axes.len() != EXPECTED_AXES.len() {
        return Err("parity report axis count is invalid".to_string());
    }
    for expected in EXPECTED_AXES {
        let axis = axes
            .iter()
            .find(|axis| axis.get("axis").and_then(Value::as_str) == Some(expected))
            .ok_or_else(|| format!("parity axis missing: {expected}"))?;
        require_bool(axis, "complete", true)?;
        require_empty_array(axis, "blocking_rows")?;
    }
    let self_build = array_field(report, "rows")?
        .iter()
        .find(|row| row.get("id").and_then(Value::as_str) == Some("crunch.self-build"))
        .ok_or("compatibility self-build row is missing")?;
    require_string(self_build, "status", "complete")?;
    require_string(self_build, "provider_kind", REQUIRED_PROVIDER_KIND)
}

fn adapter_count(bundle: &LoadedBundle, adapter_id: &str, field: &str) -> Result<u64, String> {
    array_field(value(bundle, "root-action-plan")?, "adapters")?
        .iter()
        .find(|adapter| adapter.get("adapter_id").and_then(Value::as_str) == Some(adapter_id))
        .ok_or_else(|| format!("adapter missing: {adapter_id}"))
        .and_then(|adapter| number_field(adapter, field))
}

fn read_large_member<T: DeserializeOwned>(bundle: &LoadedBundle, role: &str) -> Result<T, String> {
    let member = member(bundle, role)?;
    let path = bundle.bundle_dir.join(&member.bundle_path);
    let file = File::open(&path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let decoder =
        zstd::stream::read::Decoder::new(file).map_err(|error| format!("decode {}: {error}", path.display()))?;
    serde_json::from_reader(decoder).map_err(|error| format!("parse large member {role}: {error}"))
}

fn value<'a>(bundle: &'a LoadedBundle, role: &str) -> Result<&'a Value, String> {
    bundle.values.get(role).ok_or_else(|| format!("bundle JSON role is unavailable: {role}"))
}

fn member<'a>(bundle: &'a LoadedBundle, role: &str) -> Result<&'a BundleMember, String> {
    bundle.members.get(role).ok_or_else(|| format!("bundle member is unavailable: {role}"))
}

fn validate_expected_schema(value: &Value, expected: &str, role: &str) -> Result<(), String> {
    if expected.starts_with("json-array:") {
        if value.is_array() {
            return Ok(());
        }
        return Err(format!("member {role} is not a JSON array"));
    }
    let actual = value.get("schema").or_else(|| value.get("schema_version")).and_then(Value::as_str);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(format!("member {role} schema is {:?}, expected {expected}", actual))
    }
}

fn require_schema(value: &Value, expected: &str) -> Result<(), String> {
    validate_expected_schema(value, expected, expected)
}

fn manifest_identity(manifest: &BundleManifest) -> Result<String, String> {
    let mut value = serde_json::to_value(manifest).map_err(|error| format!("manifest identity value: {error}"))?;
    value["manifest_identity_blake3"] = Value::String(String::new());
    let bytes = serde_json::to_vec(&value).map_err(|error| format!("manifest identity bytes: {error}"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn verification_identity(receipt: &VerificationReceipt) -> Result<String, String> {
    let mut value = serde_json::to_value(receipt).map_err(|error| format!("verification identity value: {error}"))?;
    value["verification_identity_blake3"] = Value::String(String::new());
    let bytes = serde_json::to_vec(&value).map_err(|error| format!("verification identity bytes: {error}"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn read_bounded_regular(path: &Path, bytes_max: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("metadata {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || metadata.len() > bytes_max {
        return Err(format!("path is not a bounded regular no-follow file: {}", path.display()));
    }
    fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
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

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err(format!("verification output already exists: {}", path.display()));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn object_field<'a>(value: &'a Value, field: &str) -> Result<&'a Value, String> {
    let child = value.get(field).ok_or_else(|| format!("missing object field {field}"))?;
    if child.is_object() {
        Ok(child)
    } else {
        Err(format!("field {field} is not an object"))
    }
}

fn array_field<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("field {field} is not an array"))
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("field {field} is not a string"))
}

fn number_field(value: &Value, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("field {field} is not an unsigned integer"))
}

fn require_string(value: &Value, field: &str, expected: &str) -> Result<(), String> {
    require_equal(field, string_field(value, field)?, expected)
}

fn require_non_empty(value: &Value, field: &str) -> Result<(), String> {
    if string_field(value, field)?.is_empty() {
        Err(format!("field {field} is empty"))
    } else {
        Ok(())
    }
}

fn require_bool(value: &Value, field: &str, expected: bool) -> Result<(), String> {
    let actual = value.get(field).and_then(Value::as_bool).ok_or_else(|| format!("field {field} is not a boolean"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("field {field} is {actual}, expected {expected}"))
    }
}

fn require_empty_array(value: &Value, field: &str) -> Result<(), String> {
    if array_field(value, field)?.is_empty() {
        Ok(())
    } else {
        Err(format!("field {field} is not empty"))
    }
}

fn require_zero(value: &Value, field: &str) -> Result<(), String> {
    require_count(value, field, 0)
}

fn require_count(value: &Value, field: &str, expected: u64) -> Result<(), String> {
    let actual = number_field(value, field)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("field {field} is {actual}, expected {expected}"))
    }
}

fn require_equal(field: &str, actual: &str, expected: &str) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{field} is {actual}, expected {expected}"))
    }
}

fn require_exact_strings(actual: &[String], expected: &[&str], label: &str) -> Result<(), String> {
    let actual = actual.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label} differ from the required set"))
    }
}

fn require_blake3(value: &str, label: &str) -> Result<(), String> {
    require_hex(value, BLAKE3_HEX_LENGTH, label)
}

fn require_hex(value: &str, length: usize, label: &str) -> Result<(), String> {
    if value.len() == length && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        Ok(())
    } else {
        Err(format!("{label} is not lowercase hexadecimal with length {length}"))
    }
}

fn validate_relative_path(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > PATH_BYTES_MAX {
        return Err("relative path is empty or too long".to_string());
    }
    let path = Path::new(value);
    if path.is_absolute() || !path.components().all(|component| matches!(component, Component::Normal(_))) {
        return Err(format!("path has absolute or forbidden components: {value}"));
    }
    Ok(())
}

fn validate_role(value: &str) -> Result<(), String> {
    if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Ok(())
    } else {
        Err(format!("unsafe bundle role: {value}"))
    }
}

fn self_test() -> Result<(), String> {
    let mut cases = BTreeSet::new();
    self_test_boundaries(&mut cases)?;
    self_test_event_findings(&mut cases)?;
    self_test_action_authority(&mut cases)?;
    self_test_identity_links(&mut cases)?;
    const REQUIRED_CASE_COUNT: usize = 31;
    if cases.len() != REQUIRED_CASE_COUNT {
        return Err(format!("self-test case count is {}, expected {REQUIRED_CASE_COUNT}", cases.len()));
    }
    println!("source-built parity promotion self-test: PASS");
    println!("  negative cases: {}", cases.into_iter().collect::<Vec<_>>().join(","));
    Ok(())
}

fn self_test_boundaries(cases: &mut BTreeSet<String>) -> Result<(), String> {
    expect_failure("absolute-path", || validate_relative_path("/tmp/evidence.json"), cases)?;
    expect_failure("parent-path", || validate_relative_path("bundle/../evidence.json"), cases)?;
    expect_failure("malformed-digest", || require_blake3("ABC", "fixture"), cases)?;
    expect_failure("malformed-schema", || require_schema(&serde_json::json!({"schema":"wrong"}), "expected"), cases)?;
    expect_failure("witness-quorum", || validate_manifest(&synthetic_manifest("selected", 1)), cases)
}

fn self_test_event_findings(cases: &mut BTreeSet<String>) -> Result<(), String> {
    for (label, field) in [
        ("remote-execution", "remote_event_count"),
        ("cache-only", "cache_only_completion_count"),
        ("unknown-event", "unknown_event_count"),
        ("missing-action", "missing_action_count"),
        ("authority-violation", "authority_violation_count"),
    ] {
        expect_failure(label, || require_zero_findings(&finding_fixture(field)), cases)?;
    }
    expect_failure("fallback-event", || validate_audit_events(&[synthetic_event("fallback")], Some(1)), cases)?;
    expect_failure("event-count-overflow", || validate_audit_events(&[], Some(EVENT_COUNT_MAX + 1)), cases)
}

fn self_test_action_authority(cases: &mut BTreeSet<String>) -> Result<(), String> {
    expect_failure(
        "path-only-generated-authority",
        || {
            validate_fixed_entry(&serde_json::json!({
                "authority_id":"fixed:test","producer_action_id":"producer","path":"/tmp/tool","digest_blake3":""
            }))
        },
        cases,
    )?;
    expect_failure(
        "producerless-executable",
        || {
            validate_fixed_entry(&serde_json::json!({
                "authority_id":"fixed:test","producer_action_id":"","path":"/tmp/tool",
                "digest_blake3":"0".repeat(BLAKE3_HEX_LENGTH)
            }))
        },
        cases,
    )?;
    expect_failure(
        "producer-drift",
        || {
            validate_fixed_authority(&serde_json::json!({
                "producer_action_ids":["producer-a"],
                "fixed_executables":[{
                    "authority_id":"fixed:test","producer_action_id":"producer-b",
                    "path":"/tmp/tool","digest_blake3":"0".repeat(BLAKE3_HEX_LENGTH)
                }]
            }))
        },
        cases,
    )?;
    expect_failure(
        "incomplete-adapter",
        || validate_reconciliation_counts(&serde_json::json!({"action_count":2}), &reconciliation_fixture()),
        cases,
    )?;
    expect_failure(
        "target-authority",
        || require_bool(&serde_json::json!({"target_authority_excluded":false}), "target_authority_excluded", true),
        cases,
    )
}

fn self_test_identity_links(cases: &mut BTreeSet<String>) -> Result<(), String> {
    expect_failure("missing-row", || require_test_role(&BTreeSet::new(), "row-early-binutils"), cases)?;
    expect_failure("missing-action-plan", || require_test_role(&BTreeSet::new(), "root-action-plan"), cases)?;
    for (label, field, actual, expected) in [
        ("cross-row-substitution", "row", "gcc.4.0", "binutils.tcc"),
        ("scaffold-stagex", "StageX status", "scaffold-only", "complete"),
        ("v1-fixed-point", "proof schema", "mantle-deterministic-proof-receipt-v1", REQUIRED_PROOF_SCHEMA),
        ("tampered-action-plan", "action plan digest", "0", "1"),
        ("digest-drift", "digest", "a", "b"),
        ("wrong-provider", "provider", "prebuilt", REQUIRED_PROVIDER_KIND),
        ("wrong-closure", "closure", "old", "current"),
        ("wrong-source", "source", "old", "current"),
        ("stale-commit", "source commit", "old", "current"),
    ] {
        expect_failure(label, || require_equal(field, actual, expected), cases)?;
    }
    expect_failure(
        "unapproved-effect",
        || require_empty_array(&serde_json::json!({"effects":["network"]}), "effects"),
        cases,
    )?;
    expect_failure(
        "unapproved-read",
        || require_empty_array(&serde_json::json!({"reads":["ambient"]}), "reads"),
        cases,
    )?;
    expect_failure("witness-derived-status", || validate_witness_independence("derived-from-witness", 0), cases)
}

fn finding_fixture(nonzero_field: &str) -> Value {
    let mut value = serde_json::json!({
        "unknown_event_count":0,"missing_action_count":0,"authority_violation_count":0,
        "fallback_event_count":0,"remote_event_count":0,"cache_only_completion_count":0
    });
    value[nonzero_field] = Value::from(1_u64);
    value
}

fn reconciliation_fixture() -> Value {
    serde_json::json!({
        "planned_action_count":1,"matched_action_count":1,
        "observed_event_count":1,"matched_event_count":1,
        "local_only":true,"blockers":[]
    })
}

fn validate_witness_independence(policy: &str, sidecar_count: u32) -> Result<(), String> {
    if policy == "not-selected" && sidecar_count == 0 {
        Ok(())
    } else {
        Err("bootstrap promotion must not select or satisfy witness quorum".to_string())
    }
}

fn require_test_role(roles: &BTreeSet<String>, required: &str) -> Result<(), String> {
    if roles.contains(required) {
        Ok(())
    } else {
        Err(format!("required test role is missing: {required}"))
    }
}

fn expect_failure<F>(label: &str, check: F, cases: &mut BTreeSet<String>) -> Result<(), String>
where F: FnOnce() -> Result<(), String> {
    if check().is_ok() {
        return Err(format!("negative self-test accepted {label}"));
    }
    cases.insert(label.to_string());
    Ok(())
}

fn synthetic_event(decision: &str) -> AuditEvent {
    AuditEvent {
        digest_hex: "0".repeat(BLAKE3_HEX_LENGTH),
        phase: "protected".to_string(),
        inventory_entry_id: Some("planned:test".to_string()),
        policy_decision: decision.to_string(),
    }
}

fn synthetic_manifest(witness_policy: &str, witness_sidecar_count: u32) -> BundleManifest {
    BundleManifest {
        schema: BUNDLE_SCHEMA.to_string(),
        release_id: "test".to_string(),
        source_commit: "0".repeat(SOURCE_COMMIT_HEX_LENGTH),
        source_blake3: "0".repeat(BLAKE3_HEX_LENGTH),
        selected_provider_kind: REQUIRED_PROVIDER_KIND.to_string(),
        parity_axes: EXPECTED_AXES.iter().map(|value| (*value).to_string()).collect(),
        witness_policy: witness_policy.to_string(),
        witness_sidecar_count,
        members: Vec::new(),
        non_claims: REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
        manifest_identity_blake3: "0".repeat(BLAKE3_HEX_LENGTH),
    }
}
