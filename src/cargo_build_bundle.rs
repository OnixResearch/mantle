//! Filesystem composition of existing source, native-plan, receipt and signed archive formats.
//! This module does not resolve packages, grant trust, or execute a second scheduler.
// machine-artifact-public: cargo-build-bundle.manifest-artifacts
// machine-artifact-public: cargo-build-bundle.verify-report
// machine-artifact-public: cargo-build-bundle.replay-report
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use mantle_cargo_build_bundle_core::Artifact;
use mantle_cargo_build_bundle_core::Compatibility;
use mantle_cargo_build_bundle_core::MAX_MANIFEST_BYTES;
use mantle_cargo_build_bundle_core::Manifest;
use serde::Serialize;

use crate::errors::RunError;
use crate::portable_receipt::ReceiptBundle;
use crate::portable_receipt::TrustVerificationContext;
use crate::source_bundle::SourceBundleManifest;

const MAX_SMALL_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TRANSPORT_BYTES: u64 = 1_099_511_627_776;
pub const MANIFEST_FILE: &str = "cargo-build-bundle.json";

#[derive(Debug, Serialize)]
pub struct BundleReport {
    pub schema: &'static str,
    pub bundle_blake3: String,
    pub canonical_plan_blake3: String,
    pub planning_provenance: mantle_cargo_build_bundle_core::Provenance,
    pub source_payloads_verified: bool,
    pub native_plan_structurally_valid: bool,
    pub native_plan_admitted: bool,
    pub trusted_cached_results_verified: bool,
    pub execution_observed: bool,
    pub planner_rerun: bool,
    pub receipt_evidence_complete: bool,
    pub non_claim: &'static str,
}

pub struct VerifiedBundle {
    manifest: Manifest,
    pub(crate) sources: SourceBundleManifest,
    pub(crate) receipts: ReceiptBundle,
    report: BundleReport,
    goals: serde_json::Value,
}

impl VerifiedBundle {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn report(&self) -> &BundleReport {
        &self.report
    }
    pub fn into_report(self) -> BundleReport {
        self.report
    }
}

fn fail(message: impl Into<String>) -> RunError {
    RunError::Internal(message.into())
}
fn adapter(error: impl std::fmt::Display) -> RunError {
    fail(error.to_string())
}

pub fn inspect_bundle(directory: &Path, expected: &Compatibility) -> Result<Manifest, RunError> {
    let manifest: Manifest =
        serde_json::from_slice(&read_bounded(&directory.join(MANIFEST_FILE), MAX_MANIFEST_BYTES as u64)?)
            .map_err(adapter)?;
    mantle_cargo_build_bundle_core::verify(&manifest, expected).map_err(adapter)?;
    Ok(manifest)
}

/// All payload bytes and all archive signatures are verified before caller state is touched.
pub async fn verify_bundle(
    directory: &Path,
    expected: &Compatibility,
    trust: &TrustVerificationContext,
) -> Result<VerifiedBundle, RunError> {
    let manifest = inspect_bundle(directory, expected)?;
    for artifact in manifest.artifacts.values().chain(manifest.provenance.evidence.values()) {
        verify_artifact(directory, artifact)?;
    }
    let sources = crate::source_bundle::read_source_bundle(&artifact_path(directory, &manifest, "source")?)?;
    let receipts = crate::portable_receipt::read_receipt_bundle(&artifact_path(directory, &manifest, "receipts")?)?;
    if sources.manifest_blake3 != manifest.source_manifest_blake3
        || receipts.bundle_blake3 != manifest.receipt_bundle_blake3
        || sources.store_prefix != expected.store_prefix
        || receipts.store_prefix != expected.store_prefix
    {
        return Err(fail("cargo-bundle-component-identity-mismatch"));
    }
    verify_source_bindings(&manifest, &sources)?;
    verify_plan(directory, &manifest)?;
    verify_config(directory, &manifest)?;
    let goals = read_goals(directory, &manifest)?;
    let mut archive_file =
        tokio::fs::File::open(artifact_path(directory, &manifest, "store")?).await.map_err(adapter)?;
    let archive = crunch_store::list_store_archive(&mut archive_file).await.map_err(adapter)?;
    verify_cached_artifact(directory, &manifest, &archive, "plan", &manifest.producer_outputs["plan"])?;
    let config_path = manifest
        .declared_env
        .get("MANTLE_UNIT_CONFIG")
        .ok_or_else(|| fail("cargo-bundle-config-binding-missing"))?;
    verify_cached_artifact(directory, &manifest, &archive, "config", config_path)?;
    verify_planning_evidence(directory, &manifest, &archive)?;
    let outputs = verify_archive_coverage(&manifest, &archive)?;
    let receipt_report = crate::portable_receipt::verify_receipt_bundle_against_archive_and_sources(
        &receipts,
        &archive,
        &sources,
        &expected.store_prefix,
        trust,
        &outputs,
    )?;
    let report = BundleReport {
        schema: "mantle-cargo-build-bundle-report-v1",
        bundle_blake3: manifest.bundle_blake3.clone(),
        canonical_plan_blake3: manifest.canonical_plan_blake3.clone(),
        planning_provenance: manifest.provenance.clone(),
        source_payloads_verified: true,
        native_plan_structurally_valid: true,
        native_plan_admitted: false,
        trusted_cached_results_verified: true,
        execution_observed: false,
        planner_rerun: false,
        receipt_evidence_complete: receipt_report.evidence_complete,
        non_claim: mantle_cargo_build_bundle_core::NON_CLAIM,
    };
    Ok(VerifiedBundle {
        manifest,
        sources,
        receipts,
        report,
        goals,
    })
}

/// Export copies existing transports, not live checkout/Cargo/target/resultDB state.
/// Input paths are shell-only; only declared relative names participate in identity.
pub async fn export_bundle(
    manifest: Manifest,
    inputs: &BTreeMap<String, PathBuf>,
    output: &Path,
    trust: &TrustVerificationContext,
) -> Result<BundleReport, RunError> {
    let manifest = mantle_cargo_build_bundle_core::canonicalize(manifest).map_err(adapter)?;
    if output.exists() {
        return Err(fail("cargo-bundle-export-target-exists"));
    }
    let parent = output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let stage = tempfile::Builder::new().prefix("cargo-bundle-").tempdir_in(parent).map_err(adapter)?;
    for (key, artifact) in manifest.artifacts.iter().chain(manifest.provenance.evidence.iter()) {
        let input = inputs.get(key).ok_or_else(|| fail(format!("cargo-bundle-export-input-missing:{key}")))?;
        let target = stage.path().join(&artifact.file);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(adapter)?;
        }
        let source = File::open(input).map_err(adapter)?;
        if source.metadata().map_err(adapter)?.len() != artifact.bytes || artifact.bytes > MAX_TRANSPORT_BYTES {
            return Err(fail("cargo-bundle-export-input-size-mismatch"));
        }
        let mut destination = OpenOptions::new().write(true).create_new(true).open(target).map_err(adapter)?;
        let copied =
            std::io::copy(&mut source.take(artifact.bytes.saturating_add(1)), &mut destination).map_err(adapter)?;
        if copied != artifact.bytes {
            return Err(fail("cargo-bundle-export-input-size-mismatch"));
        }
        destination.sync_all().map_err(adapter)?;
    }
    write_new(
        &stage.path().join(MANIFEST_FILE),
        &mantle_cargo_build_bundle_core::encode(&manifest).map_err(adapter)?,
    )?;
    let verified = verify_bundle(stage.path(), &manifest.compatibility, trust).await?;
    crate::linux_rename::rename_path_no_replace(stage.path(), output).map_err(adapter)?;
    Ok(verified.report)
}

/// Prepare an already verified exported graph for the ordinary `mantle build` lane.
/// Caller imports the exact signed archive using `store archive import` first.
/// No planner invocation or cache/resultDB synthesis is performed here.
pub fn write_replay_goals(verified: &VerifiedBundle, output: &Path) -> Result<(), RunError> {
    // Nickel's JSON decoder yields strings, which CrunchDerivation admits for enum-like fields.
    // Serialize the JSON as a Nickel string with interpolation escaped explicitly.
    let json = serde_json::to_string(&verified.goals).map_err(adapter)?;
    let literal = serde_json::to_string(&json).map_err(adapter)?.replace("%{", "\\%{");
    write_new(output, format!("std.deserialize 'Json {literal}\n").as_bytes())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path).map_err(adapter)?;
    file.write_all(bytes).map_err(adapter)?;
    file.sync_all().map_err(adapter)
}

fn artifact_path(directory: &Path, manifest: &Manifest, key: &str) -> Result<PathBuf, RunError> {
    let artifact = manifest.artifacts.get(key).ok_or_else(|| fail(format!("cargo-bundle-artifact-missing:{key}")))?;
    Ok(directory.join(&artifact.file))
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, RunError> {
    let file = File::open(path).map_err(adapter)?;
    if file.metadata().map_err(adapter)?.len() > limit {
        return Err(fail("cargo-bundle-read-limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes).map_err(adapter)?;
    if u64::try_from(bytes.len()).map_err(adapter)? > limit {
        return Err(fail("cargo-bundle-read-limit"));
    }
    Ok(bytes)
}

fn verify_artifact(directory: &Path, artifact: &Artifact) -> Result<(), RunError> {
    let path = directory.join(&artifact.file);
    let mut current = directory.to_path_buf();
    for component in Path::new(&artifact.file).components() {
        current.push(component);
        if fs::symlink_metadata(&current).map_err(adapter)?.file_type().is_symlink() {
            return Err(fail("cargo-bundle-artifact-symlink"));
        }
    }
    let mut file = File::open(&path).map_err(adapter)?;
    if !file.metadata().map_err(adapter)?.is_file()
        || file.metadata().map_err(adapter)?.len() != artifact.bytes
        || artifact.bytes > MAX_TRANSPORT_BYTES
    {
        return Err(fail("cargo-bundle-artifact-size-mismatch"));
    }
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65_536];
    let mut total = 0u64;
    loop {
        let bytes = file.read(&mut buffer).map_err(adapter)?;
        if bytes == 0 {
            break;
        }
        total = total.checked_add(bytes as u64).ok_or_else(|| fail("cargo-bundle-artifact-limit"))?;
        if total > artifact.bytes {
            return Err(fail("cargo-bundle-artifact-size-mismatch"));
        }
        hasher.update(&buffer[..bytes]);
    }
    if total != artifact.bytes || hasher.finalize().to_hex().as_str() != artifact.blake3 {
        return Err(fail("cargo-bundle-artifact-tampered"));
    }
    Ok(())
}

fn verify_source_bindings(manifest: &Manifest, sources: &SourceBundleManifest) -> Result<(), RunError> {
    let ids = sources.records.iter().map(|record| record.identity.as_str()).collect::<BTreeSet<_>>();
    for identity in manifest.package_sources.values().chain(manifest.registry_git_sources.values()) {
        if !ids.contains(identity.as_str()) {
            return Err(fail("cargo-bundle-source-payload-missing"));
        }
    }
    Ok(())
}

fn verify_plan(directory: &Path, manifest: &Manifest) -> Result<(), RunError> {
    use crunch_build::dynamic_plan::DynamicInput;
    use crunch_build::dynamic_plan::decode_validated_plan_v2;
    let bytes = read_bounded(&artifact_path(directory, manifest, "plan")?, manifest.compatibility.max_plan_bytes)?;
    let canonical = decode_validated_plan_v2(&bytes, &manifest.compatibility.store_prefix).map_err(adapter)?;
    let plan = &canonical.plan;
    if canonical.digest.as_str() != manifest.canonical_plan_blake3 {
        return Err(fail("cargo-bundle-plan-identity-mismatch"));
    }
    let roots = plan.roots.iter().map(|id| id.as_str()).collect::<BTreeSet<_>>();
    if roots != manifest.roots.iter().map(String::as_str).collect() || plan.units.len() != manifest.units.len() {
        return Err(fail("cargo-bundle-plan-graph-mismatch"));
    }
    for unit in &plan.units {
        let declared = manifest.units.get(unit.id.as_str()).ok_or_else(|| fail("cargo-bundle-plan-unit-missing"))?;
        let deps = unit
            .derivation
            .inputs
            .iter()
            .filter_map(|input| match input {
                DynamicInput::UnitOutput { unit, .. } => Some(unit.as_str()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        if deps != declared.dependencies.iter().map(String::as_str).collect() {
            return Err(fail("cargo-bundle-plan-dependency-mismatch"));
        }
        if unit.derivation.system != manifest.compatibility.system {
            return Err(fail("cargo-bundle-plan-target-mismatch"));
        }
        if unit.derivation.builder.as_str() != manifest.tools["helper"]
            || !unit.derivation.dynamic_plan_outputs.is_empty()
        {
            return Err(fail("cargo-bundle-unit-builder-unsupported"));
        }
        if declared.package_id.is_none() && unit.derivation.args.as_slice() != ["target-facts"] {
            return Err(fail("cargo-bundle-auxiliary-unit-unsupported"));
        }
        for input in &unit.derivation.inputs {
            if let DynamicInput::StorePath { path } = input {
                require_store_payload(path.as_str(), manifest)?;
            }
        }
    }
    Ok(())
}

fn verify_config(directory: &Path, m: &Manifest) -> Result<(), RunError> {
    let bytes = read_bounded(&artifact_path(directory, m, "config")?, MAX_SMALL_ARTIFACT_BYTES)?;
    let config: serde_json::Value = serde_json::from_slice(&bytes).map_err(adapter)?;
    for (field, expected) in [
        ("store_prefix", &m.compatibility.store_prefix),
        ("system", &m.compatibility.system),
        ("host_triple", &m.compatibility.host_triple),
        ("target_triple", &m.compatibility.target_triple),
        ("profile", &m.profile),
    ] {
        if config.get(field).and_then(|v| v.as_str()) != Some(expected.as_str()) {
            return Err(fail(format!("cargo-bundle-config-mismatch:{field}")));
        }
    }
    for (field, expected) in &m.tools {
        if field == "producer" {
            continue;
        }
        if config.get(field).and_then(|v| v.as_str()) != Some(expected.as_str()) {
            return Err(fail(format!("cargo-bundle-tool-mismatch:{field}")));
        }
    }
    let native: BTreeMap<String, mantle_rust_plan_core::NativeInputDeclaration> =
        serde_json::from_value(config.get("native_inputs").cloned().unwrap_or_else(|| serde_json::json!({})))
            .map_err(adapter)?;
    if native != m.native_inputs.iter().map(|(id, item)| (id.clone(), item.declaration.clone())).collect() {
        return Err(fail("cargo-bundle-native-declaration-mismatch"));
    }
    if config.get("vendor").and_then(|v| v.as_str()) != m.vendor_store_path.as_deref() {
        return Err(fail("cargo-bundle-vendor-identity-mismatch"));
    }
    let mut features: Vec<String> =
        serde_json::from_value(config.get("features").cloned().unwrap_or_else(|| serde_json::json!([])))
            .map_err(adapter)?;
    features.sort();
    features.dedup();
    if features != m.selected_features
        || config.get("no_default_features").and_then(|v| v.as_bool()).unwrap_or(false) != m.no_default_features
    {
        return Err(fail("cargo-bundle-feature-selection-mismatch"));
    }
    if mantle_cargo_build_bundle_core::digest(&read_bounded(
        &artifact_path(directory, m, "lock")?,
        MAX_SMALL_ARTIFACT_BYTES,
    )?) != m.lock_blake3
    {
        return Err(fail("cargo-bundle-lock-identity-mismatch"));
    }
    Ok(())
}

fn read_goals(directory: &Path, m: &Manifest) -> Result<serde_json::Value, RunError> {
    let bytes = read_bounded(&artifact_path(directory, m, "goals")?, MAX_SMALL_ARTIFACT_BYTES)?;
    let goals: BTreeMap<String, crunch_glue::CrunchDerivation> = serde_json::from_slice(&bytes).map_err(adapter)?;
    if goals.is_empty() || goals.len() > 16 {
        return Err(fail("cargo-bundle-goal-limit"));
    }
    // Generic conversion checks every nested typed derivation and store input.
    let mut conversion = crunch_glue::ConversionCache::new(&m.compatibility.store_prefix);
    let mut producers = 0u32;
    for goal in goals.values() {
        crunch_glue::convert(goal, &mut conversion).map_err(adapter)?;
        if goal.builder == m.tools["producer"] {
            if goal.env.iter().collect::<BTreeMap<_, _>>() != m.declared_env.iter().collect::<BTreeMap<_, _>>() {
                return Err(fail("cargo-bundle-producer-environment-mismatch"));
            }
            let (_, drv) = crunch_glue::convert(goal, &mut conversion).map_err(adapter)?;
            for (name, expected) in &m.producer_outputs {
                let actual = drv.outputs.get(name).ok_or_else(|| fail("cargo-bundle-producer-output-missing"))?;
                if actual
                    .path
                    .as_ref()
                    .map(|path| path.to_absolute_path_with_prefix(&m.compatibility.store_prefix))
                    .as_ref()
                    != Some(expected)
                {
                    return Err(fail("cargo-bundle-producer-output-identity-mismatch"));
                }
            }
            producers = producers.saturating_add(1);
        }
    }
    if producers != 1 {
        return Err(fail("cargo-bundle-requires-one-explicit-producer-goal"));
    }
    for (_, _, drv, _, _, _) in conversion.iter_entries() {
        require_store_payload(&drv.builder, m)?;
        for source in &drv.input_sources {
            require_store_payload(&source.to_absolute_path_with_prefix(&m.compatibility.store_prefix), m)?;
        }
    }
    serde_json::from_slice(&bytes).map_err(adapter)
}

fn verify_archive_coverage(m: &Manifest, archive: &crunch_store::ArchiveListReport) -> Result<Vec<String>, RunError> {
    if archive.store_prefix != m.compatibility.store_prefix {
        return Err(fail("cargo-bundle-archive-prefix-mismatch"));
    }
    let paths = archive
        .paths
        .iter()
        .map(|path| format!("{}/{}", archive.store_prefix, path.store_path))
        .collect::<BTreeSet<_>>();
    for path in &m.required_store_paths {
        if !paths.contains(path) {
            return Err(fail(format!("cargo-bundle-store-payload-missing:{path}")));
        }
    }
    for path in &archive.paths {
        for reference in &path.references {
            if !paths.contains(&format!("{}/{reference}", archive.store_prefix)) {
                return Err(fail("cargo-bundle-store-closure-incomplete"));
            }
        }
    }
    // The portable receipt verifier must check signatures for the complete transport,
    // not merely the selected app or cached producer.
    Ok(paths.into_iter().collect())
}

fn verify_cached_artifact(
    directory: &Path,
    m: &Manifest,
    archive: &crunch_store::ArchiveListReport,
    artifact: &str,
    logical: &str,
) -> Result<(), RunError> {
    verify_signed_path(&artifact_path(directory, m, artifact)?, m, archive, logical)
}

fn verify_signed_path(
    physical: &Path,
    m: &Manifest,
    archive: &crunch_store::ArchiveListReport,
    logical: &str,
) -> Result<(), RunError> {
    let root = mantle_cargo_build_bundle_core::store_root(logical, &m.compatibility.store_prefix).map_err(adapter)?;
    if root != logical {
        return Err(fail("cargo-bundle-file-root-required"));
    }
    let name = logical.rsplit('/').next().ok_or_else(|| fail("cargo-bundle-file-root-required"))?;
    let record = archive
        .paths
        .iter()
        .find(|record| record.store_path == name)
        .ok_or_else(|| fail("cargo-bundle-signed-file-missing"))?;
    let observed = crunch_nar::observe_path(
        physical,
        crunch_nar::FilesystemNarRequest::new(
            nix_compat::nixhash::HashAlgo::Sha256,
            crunch_nar::CaseHackPolicy::Disabled,
            MAX_SMALL_ARTIFACT_BYTES,
        ),
    )
    .map_err(adapter)?;
    if observed.nar_size != record.nar_size
        || data_encoding::HEXLOWER.encode(observed.digest.digest_as_bytes()) != record.nar_sha256_hex
    {
        return Err(fail("cargo-bundle-signed-file-identity-mismatch"));
    }
    Ok(())
}

fn require_store_payload(path: &str, m: &Manifest) -> Result<(), RunError> {
    let root = mantle_cargo_build_bundle_core::store_root(path, &m.compatibility.store_prefix).map_err(adapter)?;
    if !m.required_store_paths.iter().any(|item| item == root) {
        return Err(fail(format!("cargo-bundle-store-payload-undeclared:{root}")));
    }
    Ok(())
}

fn verify_planning_evidence(
    directory: &Path,
    m: &Manifest,
    archive: &crunch_store::ArchiveListReport,
) -> Result<(), RunError> {
    let stage = tempfile::tempdir().map_err(adapter)?;
    for (key, artifact) in &m.provenance.evidence {
        let filename = format!("{key}.json");
        fs::copy(directory.join(&artifact.file), stage.path().join(filename)).map_err(adapter)?;
    }
    verify_signed_path(stage.path(), m, archive, &m.producer_outputs["out"])?;
    let load = |name: &str| -> Result<serde_json::Value, RunError> {
        let artifact = m.provenance.evidence.get(name).ok_or_else(|| fail("cargo-bundle-evidence-missing"))?;
        serde_json::from_slice(&read_bounded(&directory.join(&artifact.file), MAX_SMALL_ARTIFACT_BYTES)?)
            .map_err(adapter)
    };
    let graph = load("cargo-unit-graph")?;
    if graph.get("version").and_then(|v| v.as_u64()) != Some(1) {
        return Err(fail("cargo-bundle-planning-version-unsupported"));
    }
    let receipt = load("rust-plan-receipt")?;
    let graph_digest = mantle_cargo_build_bundle_core::digest(&serde_json::to_vec(&graph).map_err(adapter)?);
    if receipt.pointer("/schema_version").and_then(|v| v.as_u64()) != Some(1)
        || receipt.pointer("/unit_derivation_graph/ready").and_then(|v| v.as_bool()) != Some(true)
        || receipt.pointer("/cargo_mode/compatibility_class").and_then(|v| v.as_str()) != Some("cargo-oracle-evidence")
        || receipt.pointer("/lockfile/blake3").and_then(|v| v.as_str()) != Some(m.lock_blake3.as_str())
        || receipt.pointer("/unit_graph/digest_blake3").and_then(|v| v.as_str()) != Some(graph_digest.as_str())
    {
        return Err(fail("cargo-bundle-planning-provenance-mismatch"));
    }
    let bindings = load("cargo-unit-bindings")?;
    if bindings.get("schema").and_then(|v| v.as_str()) != Some("mantle-cargo-unit-bindings-v1") {
        return Err(fail("cargo-bundle-unit-bindings-schema-unsupported"));
    }
    let units = bindings
        .get("units")
        .and_then(|v| v.as_object())
        .ok_or_else(|| fail("cargo-bundle-unit-bindings-missing"))?;
    for (id, unit) in &m.units {
        if let Some(package) = &unit.package_id {
            let binding = units.get(id).ok_or_else(|| fail("cargo-bundle-unit-binding-missing"))?;
            let features: Vec<String> = serde_json::from_value(
                binding.get("features").cloned().ok_or_else(|| fail("cargo-bundle-unit-features-missing"))?,
            )
            .map_err(adapter)?;
            if binding.get("package_id").and_then(|v| v.as_str()) != Some(package.as_str()) || features != unit.features
            {
                return Err(fail("cargo-bundle-unit-binding-mismatch"));
            }
        } else if units.contains_key(id) {
            return Err(fail("cargo-bundle-auxiliary-unit-binding-invalid"));
        }
    }
    let summary = load("cargo-unit-plan-evidence")?;
    if summary.get("schema").and_then(|v| v.as_str()) != Some("mantle-cargo-unit-plan-evidence-v1")
        || summary.get("lock_blake3").and_then(|v| v.as_str()) != Some(m.lock_blake3.as_str())
        || summary.get("unit_graph_blake3").and_then(|v| v.as_str()) != Some(graph_digest.as_str())
        || summary.get("plan_blake3").and_then(|v| v.as_str()) != Some(m.artifacts["plan"].blake3.as_str())
    {
        return Err(fail("cargo-bundle-producer-summary-mismatch"));
    }
    let metadata = load("cargo-metadata")?;
    let packages = metadata
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| fail("cargo-bundle-metadata-packages-missing"))?;
    for package in packages {
        let id = package
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| fail("cargo-bundle-metadata-package-id-missing"))?;
        if m.package_sources.contains_key(id)
            && package.get("source").is_some_and(|v| !v.is_null())
            && m.registry_git_sources.get(id) != m.package_sources.get(id)
        {
            return Err(fail("cargo-bundle-registry-git-source-binding-missing"));
        }
    }
    if units.keys().any(|id| !m.units.contains_key(id)) {
        return Err(fail("cargo-bundle-unit-binding-extra"));
    }
    Ok(())
}
