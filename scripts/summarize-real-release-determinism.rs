#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "1.8.2"
serde_json = "1"
---

//! Summarize a validated real release-specific deterministic proof rail output.
//!
//! This script first delegates contract validation to
//! `scripts/check-real-release-determinism-receipt.rs`, then writes portable JSON
//! and Markdown summary artifacts for archival/review. The summary is bounded to
//! the packaged release artifact proof and does not claim full bootstrap
//! reproducibility.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde_json::Value;
use serde_json::json;

const SUMMARY_SCHEMA: &str = "mantle-real-release-determinism-summary-v1";
const BOUNDED_CLAIM: &str = "This artifact rebuilt twice from the exact source closure, recipe, executable/tool identities, provider identity, policies, and fresh run roots bound by the v2 descriptor/authority plan under recorded mantle-proof-sandbox-v1 profiles, and the BLAKE3 digest sets matched. This does not claim compiler/verifier soundness or full-bootstrap reproducibility.";
const PROOF_VERDICT: &str = "self-rebuild-match";
const VERIFY_STATUS: &str = "eligible";
const SANDBOX_PREFIX: &str = "mantle-proof-sandbox-v1:";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(env::args().skip(1))?;
    if args.help {
        print_usage();
        return Ok(());
    }
    if args.self_test {
        return run_self_test();
    }
    let release_bundle =
        args.release_bundle.as_deref().ok_or("release bundle path is required unless --self-test is used")?;
    let paths = ProofPaths::resolve(release_bundle, args.proof_dir.as_deref(), args.verify_receipt.as_deref())?;
    run_validator(release_bundle, args.proof_dir.as_deref(), args.verify_receipt.as_deref())?;
    let summary = build_summary(&paths)?;
    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| release_bundle.parent().unwrap_or_else(|| Path::new(".")).to_path_buf());
    fs::create_dir_all(&output_dir).map_err(|err| format!("create {}: {err}", output_dir.display()))?;
    let release_id = field_str(&summary, "release_id")?;
    let json_path = output_dir.join(format!("{release_id}-determinism-summary.json"));
    let md_path = output_dir.join(format!("{release_id}-determinism-summary.md"));
    write_json(&json_path, &summary)?;
    fs::write(&md_path, render_markdown(&summary)?).map_err(|err| format!("write {}: {err}", md_path.display()))?;
    println!("real release determinism summary written");
    println!("  json: {}", json_path.display());
    println!("  markdown: {}", md_path.display());
    println!("  bounded claim: {BOUNDED_CLAIM}");
    Ok(())
}

#[derive(Debug)]
struct Args {
    release_bundle: Option<PathBuf>,
    proof_dir: Option<PathBuf>,
    verify_receipt: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    self_test: bool,
    help: bool,
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where I: Iterator<Item = String> {
        let mut parsed = Args {
            release_bundle: None,
            proof_dir: None,
            verify_receipt: None,
            output_dir: None,
            self_test: false,
            help: false,
        };
        let mut args = args.peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--proof-dir" => {
                    parsed.proof_dir = Some(PathBuf::from(args.next().ok_or("--proof-dir requires a value")?))
                }
                "--verify-receipt" => {
                    parsed.verify_receipt =
                        Some(PathBuf::from(args.next().ok_or("--verify-receipt requires a value")?));
                }
                "--output-dir" => {
                    parsed.output_dir = Some(PathBuf::from(args.next().ok_or("--output-dir requires a value")?))
                }
                "--self-test" => parsed.self_test = true,
                "-h" | "--help" => parsed.help = true,
                other if other.starts_with('-') => return Err(format!("unknown argument: {other}")),
                path => {
                    if parsed.release_bundle.is_some() {
                        return Err(format!("unexpected extra argument: {path}"));
                    }
                    parsed.release_bundle = Some(PathBuf::from(path));
                }
            }
        }
        Ok(parsed)
    }
}

fn print_usage() {
    println!(
        "Usage: cargo -Zscript scripts/summarize-real-release-determinism.rs [--proof-dir DIR] [--verify-receipt JSON] [--output-dir DIR] RELEASE_BUNDLE"
    );
    println!("       cargo -Zscript scripts/summarize-real-release-determinism.rs --self-test");
}

#[derive(Debug)]
struct ProofPaths {
    release_bundle: PathBuf,
    manifest: PathBuf,
    proof_dir: PathBuf,
    proof: PathBuf,
    sandbox: PathBuf,
    verify: PathBuf,
}

impl ProofPaths {
    fn resolve(release_bundle: &Path, proof_dir: Option<&Path>, verify_receipt: Option<&Path>) -> Result<Self, String> {
        let release_bundle = release_bundle.to_path_buf();
        let release_id = release_bundle
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("release bundle has no usable directory name: {}", release_bundle.display()))?;
        let parent = release_bundle.parent().unwrap_or_else(|| Path::new("."));
        let proof_dir = proof_dir.map(Path::to_path_buf).unwrap_or_else(|| parent.join(format!("{release_id}-proof")));
        let verify = verify_receipt
            .map(Path::to_path_buf)
            .unwrap_or_else(|| parent.join(format!("verify-{release_id}.json")));
        Ok(Self {
            manifest: release_bundle.join("manifest.json"),
            proof: proof_dir.join("deterministic-build-proof.json"),
            sandbox: proof_dir.join("deterministic-sandbox-isolation-evidence.json"),
            release_bundle,
            proof_dir,
            verify,
        })
    }
}

fn run_validator(release_bundle: &Path, proof_dir: Option<&Path>, verify_receipt: Option<&Path>) -> Result<(), String> {
    let validator = Path::new("scripts/check-real-release-determinism-receipt.rs");
    if !validator.exists() {
        return Err(format!("validator script not found: {}", validator.display()));
    }
    let mut command = Command::new("cargo");
    command.arg("-Zscript").arg(validator);
    if let Some(proof_dir) = proof_dir {
        command.arg("--proof-dir").arg(proof_dir);
    }
    if let Some(verify_receipt) = verify_receipt {
        command.arg("--verify-receipt").arg(verify_receipt);
    }
    command.arg(release_bundle);
    let output = command.output().map_err(|err| format!("run validator: {err}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("validator failed with status {}\nstdout:\n{}\nstderr:\n{}", output.status, stdout, stderr))
    }
}

fn build_summary(paths: &ProofPaths) -> Result<Value, String> {
    let manifest = read_json(&paths.manifest)?;
    let proof = read_json(&paths.proof)?;
    let sandbox = read_json(&paths.sandbox)?;
    let verify = read_json(&paths.verify)?;

    let release_id = field_str(&manifest, "release_id")?;
    let deterministic = object_field(&verify, "deterministic_release")?;
    let runs = array_field(&proof, "runs")?;
    let artifact_digests = artifact_digests(runs.first().ok_or("proof.runs must not be empty")?)?;
    let sandbox_profiles = strings_array(&proof, "sandbox_profile_identities")?;
    for (idx, profile) in sandbox_profiles.iter().enumerate() {
        if !profile.starts_with(SANDBOX_PREFIX) {
            return Err(format!("sandbox_profile_identities[{idx}] missing {SANDBOX_PREFIX} prefix"));
        }
    }

    Ok(json!({
        "schema": SUMMARY_SCHEMA,
        "generated_unix_ms": unix_ms(),
        "release_id": release_id,
        "release_bundle_path": paths.release_bundle,
        "proof_dir_path": paths.proof_dir,
        "verify_receipt_path": paths.verify,
        "workflow": {
            "release_manifest_schema": field_str(&manifest, "schema")?,
            "proof_workflow_version": field_str(&proof, "workflow_version")?,
            "verify_kind": field_str(&verify, "kind")?,
            "summary_schema": SUMMARY_SCHEMA
        },
        "provider": {
            "selected_provider_kind": field_str(&proof, "selected_provider_kind")?,
            "toolchain_provider_identity": field_str(&proof, "toolchain_provider_identity")?
        },
        "inputs": {
            "source_blake3": field_str(&proof, "source_blake3")?,
            "vendor_blake3": field_str(&proof, "vendor_blake3")?,
            "prerequisite_inventory_blake3": field_str(object_field(&manifest, "prerequisite_inventory")?, "digest_blake3")?,
            "toolchain_stage_roots": proof.get("toolchain_stage_roots").cloned().unwrap_or_else(|| json!([]))
        },
        "artifacts": artifact_digests,
        "sandbox": {
            "schema": field_str(&sandbox, "schema")?,
            "profile_family": field_str(&sandbox, "profile_family")?,
            "status": field_str(&sandbox, "status")?,
            "profile_identities": sandbox_profiles,
            "evidence_digest_blake3": field_str(&sandbox, "evidence_digest_blake3")?,
            "sandbox_evidence_file_blake3": blake3_file(&paths.sandbox)?
        },
        "proof": {
            "verdict": field_str(&proof, "verdict")?,
            "physical_store_isolation": field_str(&proof, "physical_store_isolation")?,
            "run_count": runs.len(),
            "run_root_identities": runs.iter().map(|run| field_str(run, "output_root_identity").map(String::from)).collect::<Result<Vec<_>, _>>()?,
            "proof_file_blake3": blake3_file(&paths.proof)?,
            "rebuild_descriptor_blake3": field_str(&proof, "rebuild_descriptor_blake3")?,
            "rebuild_authority_plan_blake3": field_str(&proof, "rebuild_authority_plan_blake3")?
        },
        "verify": {
            "status": field_str(deterministic, "status")?,
            "eligible": deterministic.get("eligible").and_then(Value::as_bool).unwrap_or(false),
            "proof_digest_blake3": field_str(deterministic, "proof_digest_blake3")?,
            "sandbox_isolation_evidence_digest_blake3": field_str(deterministic, "sandbox_isolation_evidence_digest_blake3")?,
            "verify_receipt_file_blake3": blake3_file(&paths.verify)?
        },
        "bounded_claim": BOUNDED_CLAIM,
        "non_claims": [
            "compiler or verifier soundness",
            "full bootstrap reproducibility",
            "absence of all environmental influence",
            "parity with every upstream bootstrap lineage"
        ]
    }))
}

fn render_markdown(summary: &Value) -> Result<String, String> {
    let mut out = String::new();
    let release_id = field_str(summary, "release_id")?;
    out.push_str(&format!("# Real release determinism summary: {release_id}\n\n"));
    out.push_str(&format!("**Schema:** `{}`\n\n", field_str(summary, "schema")?));
    out.push_str("## Verdict\n\n");
    out.push_str(&format!("- Proof verdict: `{}`\n", field_str(object_field(summary, "proof")?, "verdict")?));
    out.push_str(&format!("- Verify status: `{}`\n", field_str(object_field(summary, "verify")?, "status")?));
    out.push_str(&format!("- Bounded claim: {}\n\n", field_str(summary, "bounded_claim")?));

    out.push_str("## Inputs\n\n");
    let provider = object_field(summary, "provider")?;
    let inputs = object_field(summary, "inputs")?;
    out.push_str(&format!("- Provider kind: `{}`\n", field_str(provider, "selected_provider_kind")?));
    out.push_str(&format!("- Source BLAKE3: `{}`\n", field_str(inputs, "source_blake3")?));
    out.push_str(&format!("- Vendor/proof bundle BLAKE3: `{}`\n", field_str(inputs, "vendor_blake3")?));
    out.push_str(&format!(
        "- Prerequisite inventory BLAKE3: `{}`\n\n",
        field_str(inputs, "prerequisite_inventory_blake3")?
    ));

    out.push_str("## Artifact digest set\n\n");
    for artifact in array_field(summary, "artifacts")? {
        out.push_str(&format!(
            "- `{}` — BLAKE3 `{}`\n",
            field_str(artifact, "name")?,
            field_str(artifact, "digest_blake3")?
        ));
    }
    out.push('\n');

    out.push_str("## Evidence files\n\n");
    let sandbox = object_field(summary, "sandbox")?;
    let proof = object_field(summary, "proof")?;
    let verify = object_field(summary, "verify")?;
    out.push_str(&format!("- Rebuild descriptor BLAKE3: `{}`\n", field_str(proof, "rebuild_descriptor_blake3")?));
    out.push_str(&format!(
        "- Rebuild authority plan BLAKE3: `{}`\n",
        field_str(proof, "rebuild_authority_plan_blake3")?
    ));
    out.push_str(&format!("- Proof file BLAKE3: `{}`\n", field_str(proof, "proof_file_blake3")?));
    out.push_str(&format!(
        "- Sandbox evidence file BLAKE3: `{}`\n",
        field_str(sandbox, "sandbox_evidence_file_blake3")?
    ));
    out.push_str(&format!("- Verify proof digest BLAKE3: `{}`\n", field_str(verify, "proof_digest_blake3")?));
    out.push_str(&format!(
        "- Verify sandbox evidence digest BLAKE3: `{}`\n",
        field_str(verify, "sandbox_isolation_evidence_digest_blake3")?
    ));
    out.push_str(&format!("- Verify receipt file BLAKE3: `{}`\n\n", field_str(verify, "verify_receipt_file_blake3")?));

    out.push_str("## Sandbox profiles\n\n");
    for profile in array_field(sandbox, "profile_identities")? {
        out.push_str(&format!("- `{}`\n", profile.as_str().ok_or("sandbox profile is not a string")?));
    }
    out.push('\n');

    out.push_str("## Non-claims\n\n");
    for non_claim in array_field(summary, "non_claims")? {
        out.push_str(&format!("- {}\n", non_claim.as_str().ok_or("non_claim is not a string")?));
    }
    Ok(out)
}

fn artifact_digests(run: &Value) -> Result<Vec<Value>, String> {
    let mut artifacts = Vec::new();
    for digest in array_field(run, "output_digests")? {
        artifacts.push(json!({
            "name": field_str(digest, "name")?,
            "digest_blake3": field_str(digest, "digest_blake3")?
        }));
    }
    if artifacts.is_empty() {
        Err("output_digests must not be empty".to_string())
    } else {
        Ok(artifacts)
    }
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} as JSON: {err}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|err| format!("serialize JSON: {err}"))?;
    fs::write(path, format!("{text}\n")).map_err(|err| format!("write {}: {err}", path.display()))
}

fn blake3_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn object_field<'a>(value: &'a Value, field: &str) -> Result<&'a Value, String> {
    value
        .get(field)
        .filter(|value| value.is_object())
        .ok_or_else(|| format!("missing object field {field}"))
}

fn array_field<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing array field {field}"))
}

fn strings_array(value: &Value, field: &str) -> Result<Vec<String>, String> {
    array_field(value, field)?
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            item.as_str().map(ToString::to_string).ok_or_else(|| format!("{field}[{idx}] is not a string"))
        })
        .collect()
}

fn field_str<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("missing string field {field}"))
}

fn write_fixture_via_validator(root: &Path) -> Result<(), String> {
    let validator = Path::new("scripts/check-real-release-determinism-receipt.rs");
    let output = Command::new("cargo")
        .arg("-Zscript")
        .arg(validator)
        .arg("--write-self-test-fixture")
        .arg(root)
        .output()
        .map_err(|err| format!("write validated self-test fixture: {err}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "validator fixture generation failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn run_self_test() -> Result<(), String> {
    let dir =
        env::temp_dir().join(format!("mantle-real-release-determinism-summary-{}-{}", std::process::id(), unix_ms()));
    let release_bundle = dir.join("demo-release");
    let proof_dir = dir.join("demo-release-proof");
    let output_dir = dir.join("summary-out");
    fs::create_dir_all(&output_dir).map_err(|err| format!("create {}: {err}", output_dir.display()))?;
    write_fixture_via_validator(&dir)?;
    run_validator(&release_bundle, Some(&proof_dir), Some(&dir.join("verify-demo-release.json")))?;
    let paths = ProofPaths::resolve(&release_bundle, Some(&proof_dir), Some(&dir.join("verify-demo-release.json")))?;
    let summary = build_summary(&paths)?;
    if field_str(&summary, "schema")? != SUMMARY_SCHEMA {
        return Err("self-test summary schema mismatch".to_string());
    }
    if field_str(object_field(&summary, "proof")?, "verdict")? != PROOF_VERDICT {
        return Err("self-test proof verdict mismatch".to_string());
    }
    if field_str(object_field(&summary, "verify")?, "status")? != VERIFY_STATUS {
        return Err("self-test verify status mismatch".to_string());
    }
    let md = render_markdown(&summary)?;
    for required in [
        "exact source closure",
        "compiler or verifier soundness",
        "full bootstrap reproducibility",
        "mantle-proof-sandbox-v1:",
    ] {
        if !md.contains(required) {
            return Err(format!("self-test markdown missing {required:?}"));
        }
    }
    fs::remove_dir_all(&dir).map_err(|err| format!("remove {}: {err}", dir.display()))?;
    println!("real release determinism summary self-test passed");
    Ok(())
}

fn unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}
