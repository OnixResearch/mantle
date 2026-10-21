//! CLI shell for composition; every mutation follows full offline verification.
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use clap::Args;
use clap::Subcommand;
use mantle_application_contract::rust_operations::RustEvidence;
use mantle_application_contract::rust_operations::RustOperation;
use mantle_cargo_build_bundle_core::Compatibility;
use mantle_cargo_build_bundle_core::Manifest;

use crate::cargo_build_bundle::BundleReport;
use crate::cargo_build_bundle::MANIFEST_FILE;
use crate::errors::RunError;
use crate::portable_receipt::TrustVerificationContext;

// The CLI output enum stays unboxed by design: both variants are owned for the
// duration of one command and boxing would obscure the wire-shaped surface.
#[allow(clippy::large_enum_variant)]
enum BundleCommandOutput {
    Manifest(Manifest),
    Report(BundleReport),
}

impl BundleCommandOutput {
    fn evidence(&self) -> RustEvidence<'_> {
        match self {
            Self::Manifest(manifest) => RustEvidence::BundleManifest {
                bundle_digest: &manifest.bundle_blake3,
                plan_digest: &manifest.canonical_plan_blake3,
            },
            Self::Report(report) => RustEvidence::Bundle {
                bundle_digest: &report.bundle_blake3,
                source_verified: report.source_payloads_verified,
                plan_valid: report.native_plan_structurally_valid,
                cached_results_verified: report.trusted_cached_results_verified,
                execution_observed: report.execution_observed,
                planner_rerun: report.planner_rerun,
            },
        }
    }
}

#[derive(Clone, Debug, Args)]
pub struct BundleVerificationArgs {
    #[arg(long)]
    pub compatibility: PathBuf,
    #[arg(long)]
    pub policy_hash: String,
    #[arg(long)]
    pub valid_at_unix_s: u64,
    #[arg(long)]
    pub revocation_ref: Option<String>,
    #[arg(long = "revoked-public-key-digest")]
    pub revoked_public_key_digests: Vec<String>,
    #[arg(long = "trusted-public-key", required = true)]
    pub trusted_public_keys: Vec<String>,
    /// Independently obtained canonical manifest identity; never inferred from transport.
    #[arg(long)]
    pub expected_bundle_blake3: Option<String>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CargoBuildBundleAction {
    /// Read bounded canonical metadata without importing, trusting, or executing it.
    Inspect {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        compatibility: PathBuf,
    },
    /// Verify all payloads, native admission and existing signed receipt chains without writes.
    Verify {
        #[arg(long)]
        from: PathBuf,
        #[command(flatten)]
        verification: BundleVerificationArgs,
    },
    /// Copy explicitly supplied existing transports into a canonical immutable directory.
    Export {
        #[arg(long)]
        manifest: PathBuf,
        /// JSON map of artifact/evidence keys to producer-local transport filenames.
        #[arg(long)]
        inputs: PathBuf,
        #[arg(long)]
        to: PathBuf,
        #[command(flatten)]
        verification: BundleVerificationArgs,
    },
    /// Verify first, import the existing transports, then replay the frozen graph without planning.
    Replay {
        #[arg(long)]
        from: PathBuf,
        /// Fresh directory; existing client state is deliberately refused.
        #[arg(long)]
        client_dir: PathBuf,
        #[command(flatten)]
        verification: BundleVerificationArgs,
    },
}

fn fail(message: impl Into<String>) -> RunError {
    RunError::Internal(message.into())
}
fn adapter(error: impl std::fmt::Display) -> RunError {
    fail(error.to_string())
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, RunError> {
    use std::io::Read;
    let limit = mantle_cargo_build_bundle_core::MAX_MANIFEST_BYTES as u64;
    let file = std::fs::File::open(path).map_err(adapter)?;
    if file.metadata().map_err(adapter)?.len() > limit {
        return Err(fail("cargo-bundle-command-input-limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes).map_err(adapter)?;
    if bytes.len() > mantle_cargo_build_bundle_core::MAX_MANIFEST_BYTES {
        return Err(fail("cargo-bundle-command-input-limit"));
    }
    serde_json::from_slice(&bytes).map_err(adapter)
}

pub async fn cmd_cargo_build_bundle(action: CargoBuildBundleAction) -> Result<(), RunError> {
    let operation = match &action {
        CargoBuildBundleAction::Inspect { .. } => RustOperation::InspectBundle,
        CargoBuildBundleAction::Verify { .. } => RustOperation::VerifyBundle,
        CargoBuildBundleAction::Export { .. } => RustOperation::ExportBundle,
        CargoBuildBundleAction::Replay { .. } => RustOperation::ReplayBundle,
    };
    let gate = crate::rust_plan_execution::RustOperationGate::plan(operation)?;
    let result =
        mantle_application_contract::rust_operations::RustOperationPort::execute(|| execute_cargo_build_bundle(action))
            .await;
    let classification = gate.classify_result(&result, BundleCommandOutput::evidence);
    let output = result?;
    classification?;
    match output {
        BundleCommandOutput::Manifest(manifest) => print_json(&manifest),
        BundleCommandOutput::Report(report) => print_json(&report),
    }
}

async fn execute_cargo_build_bundle(action: CargoBuildBundleAction) -> Result<BundleCommandOutput, RunError> {
    match action {
        CargoBuildBundleAction::Inspect { from, compatibility } => {
            let expected = read_json(&compatibility)?;
            Ok(BundleCommandOutput::Manifest(crate::cargo_build_bundle::inspect_bundle(&from, &expected)?))
        }
        CargoBuildBundleAction::Verify { from, verification } => {
            let (expected, trust) = verification_context(&verification)?;
            let verified = crate::cargo_build_bundle::verify_bundle(&from, &expected, &trust).await?;
            require_expected_identity(verified.manifest(), &verification)?;
            Ok(BundleCommandOutput::Report(verified.into_report()))
        }
        CargoBuildBundleAction::Export {
            manifest,
            inputs,
            to,
            verification,
        } => {
            let (expected, trust) = verification_context(&verification)?;
            let manifest: Manifest = read_json(&manifest)?;
            let manifest = mantle_cargo_build_bundle_core::canonicalize(manifest).map_err(adapter)?;
            mantle_cargo_build_bundle_core::verify(&manifest, &expected).map_err(adapter)?;
            if verification.expected_bundle_blake3.is_some() {
                require_expected_identity(&manifest, &verification)?;
            }
            let inputs: BTreeMap<String, PathBuf> = read_json(&inputs)?;
            Ok(BundleCommandOutput::Report(
                crate::cargo_build_bundle::export_bundle(manifest, &inputs, &to, &trust).await?,
            ))
        }
        CargoBuildBundleAction::Replay {
            from,
            client_dir,
            verification,
        } => {
            let (expected, trust) = verification_context(&verification)?;
            let verified = crate::cargo_build_bundle::verify_bundle(&from, &expected, &trust).await?;
            require_expected_identity(verified.manifest(), &verification)?;
            let from = std::fs::canonicalize(&from).map_err(adapter)?;
            let client_dir = std::path::absolute(&client_dir).map_err(adapter)?;
            let report = replay(&from, &client_dir, &verified, &verification)?;
            Ok(BundleCommandOutput::Report(report))
        }
    }
}

fn verification_context(args: &BundleVerificationArgs) -> Result<(Compatibility, TrustVerificationContext), RunError> {
    let expected: Compatibility = read_json(&args.compatibility)?;
    if args.policy_hash != expected.trust_policy_blake3 {
        return Err(fail("cargo-bundle-trust-policy-mismatch"));
    }
    let keys = args
        .trusted_public_keys
        .iter()
        .map(|token| nix_compat::narinfo::VerifyingKey::parse(token).map_err(adapter))
        .collect::<Result<Vec<_>, _>>()?;
    let trust = TrustVerificationContext::new(
        args.policy_hash.clone(),
        args.valid_at_unix_s,
        args.revocation_ref.clone(),
        args.revoked_public_key_digests.clone(),
    )
    .with_trusted_public_keys(keys);
    Ok((expected, trust))
}
fn require_expected_identity(m: &Manifest, args: &BundleVerificationArgs) -> Result<(), RunError> {
    if args.expected_bundle_blake3.as_deref() != Some(m.bundle_blake3.as_str()) {
        return Err(fail("cargo-bundle-expected-identity-missing-or-mismatch"));
    }
    Ok(())
}
fn print_json(value: &impl serde::Serialize) -> Result<(), RunError> {
    println!("{}", serde_json::to_string_pretty(value).map_err(adapter)?);
    Ok(())
}

fn replay(
    directory: &Path,
    client: &Path,
    verified: &crate::cargo_build_bundle::VerifiedBundle,
    args: &BundleVerificationArgs,
) -> Result<BundleReport, RunError> {
    if client.exists() {
        return Err(fail("cargo-bundle-clean-client-required"));
    }
    // This is the first caller-visible mutation; all source/plan/signature checks finished.
    std::fs::create_dir(client).map_err(adapter)?;
    let goals = client.join("replay.ncl");
    crate::cargo_build_bundle::write_replay_goals(verified, &goals)?;
    let manifest = verified.manifest();
    let exe = std::env::current_exe().map_err(adapter)?;
    let store = client.join("store");
    let state = client.join("state");
    let mut base = vec![
        "--json".to_string(),
        "--store-prefix".to_string(),
        manifest.compatibility.store_prefix.clone(),
        "--store".to_string(),
        store.to_string_lossy().into_owned(),
        "--state-dir".to_string(),
        state.to_string_lossy().into_owned(),
    ];
    let artifact = |key: &str| directory.join(&manifest.artifacts[key].file).to_string_lossy().into_owned();
    let keys = args.trusted_public_keys.join(",");
    run(
        &exe,
        &base,
        &[
            "store",
            "archive",
            "import",
            "--from",
            &artifact("store"),
            "--trusted-public-keys",
            &keys,
        ],
        client,
    )?;
    crate::source_bundle::import_source_bundle(&verified.sources, &state, true)?;
    let outputs = manifest.required_store_paths.clone();
    let (_, trust) = verification_context(args)?;
    crate::portable_receipt::import_verified_receipt_bundle(
        &verified.receipts,
        &state,
        &manifest.compatibility.store_prefix,
        &trust,
        &outputs,
    )?;
    base.extend([
        "build".to_string(),
        goals.to_string_lossy().into_owned(),
        "--no-substitute".to_string(),
        "--trusted-public-keys".to_string(),
        keys,
        "--offline-source-preflight".to_string(),
    ]);
    let bytes = run(&exe, &base, &[], client)?;
    let build: serde_json::Value = serde_json::from_slice(&bytes).map_err(adapter)?;
    std::fs::write(client.join("build-report.json"), &bytes).map_err(adapter)?;
    verify_replay_observation(manifest, &build)?;
    let report = BundleReport {
        schema: verified.report().schema,
        bundle_blake3: manifest.bundle_blake3.clone(),
        canonical_plan_blake3: manifest.canonical_plan_blake3.clone(),
        planning_provenance: manifest.provenance.clone(),
        source_payloads_verified: true,
        native_plan_structurally_valid: true,
        native_plan_admitted: true,
        trusted_cached_results_verified: true,
        execution_observed: build
            .pointer("/counts/built_total")
            .and_then(|value| value.as_u64())
            .is_some_and(|count| count > 0),
        planner_rerun: false,
        receipt_evidence_complete: verified.report().receipt_evidence_complete,
        non_claim: verified.report().non_claim,
    };
    // Keep the manifest alongside the report, not a resultDB or a replacement receipt.
    std::fs::copy(directory.join(MANIFEST_FILE), client.join(MANIFEST_FILE)).map_err(adapter)?;
    Ok(report)
}

fn run(exe: &Path, base: &[String], args: &[&str], client: &Path) -> Result<Vec<u8>, RunError> {
    let output = Command::new(exe)
        .args(base)
        .args(args)
        .current_dir(client)
        .env_clear()
        .env("HOME", client.join("home"))
        .env("CARGO_HOME", client.join("cargo-home"))
        .env("CARGO_TARGET_DIR", client.join("target"))
        .env("PATH", "/run/wrappers/bin:/run/current-system/sw/bin:/usr/bin:/bin")
        .output()
        .map_err(adapter)?;
    if !output.status.success() {
        return Err(fail(format!("cargo-bundle-replay-command-failed: {}", String::from_utf8_lossy(&output.stderr))));
    }
    Ok(output.stdout)
}

fn verify_replay_observation(m: &Manifest, build: &serde_json::Value) -> Result<(), RunError> {
    let plans = build
        .get("native_dynamic_plans")
        .and_then(|v| v.as_array())
        .ok_or_else(|| fail("cargo-bundle-replay-plan-report-missing"))?;
    let expected_ids = m.units.keys().map(String::as_str).collect::<std::collections::BTreeSet<_>>();
    if !plans.iter().any(|plan| {
        plan.get("canonical_plan_digest").and_then(|v| v.as_str()) == Some(m.canonical_plan_blake3.as_str())
            && plan.get("rejection_reason").is_some_and(serde_json::Value::is_null)
            && plan.get("accepted_unit_ids").and_then(|v| v.as_array()).is_some_and(|ids| {
                ids.iter().filter_map(|id| id.as_str()).collect::<std::collections::BTreeSet<_>>() == expected_ids
            })
    }) {
        return Err(fail("cargo-bundle-replay-plan-identity-mismatch"));
    }
    let outcomes = build
        .get("outcomes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| fail("cargo-bundle-replay-outcomes-missing"))?;
    if build.pointer("/counts/failed_total").and_then(|value| value.as_u64()) != Some(0) {
        return Err(fail("cargo-bundle-replay-execution-failed"));
    }
    if !outcomes.iter().any(|outcome| {
        outcome.get("cached").and_then(|v| v.as_bool()) == Some(true)
            && outcome.get("outputs").and_then(|v| v.as_array()).is_some_and(|outputs| {
                outputs.iter().any(|output| {
                    output.get("path").and_then(|v| v.as_str()).is_some_and(|path| {
                        m.producer_outputs
                            .values()
                            .any(|expected| path.rsplit('/').next() == expected.rsplit('/').next())
                    })
                })
            })
    }) {
        return Err(fail("cargo-bundle-replay-cached-producer-missing"));
    }
    if outcomes.iter().any(|outcome| {
        outcome.get("cached").and_then(|v| v.as_bool()) == Some(false)
            && outcome.get("outputs").and_then(|v| v.as_array()).is_some_and(|outputs| {
                outputs.iter().any(|output| {
                    output.get("path").and_then(|v| v.as_str()).is_some_and(|path| {
                        m.producer_outputs
                            .values()
                            .any(|expected| path.rsplit('/').next() == expected.rsplit('/').next())
                    })
                })
            })
    }) {
        return Err(fail("cargo-bundle-replay-planner-reran"));
    }
    Ok(())
}
