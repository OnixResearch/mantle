// machine-artifact-public: import.command-reports
//! Cargo acquisition is an explicit stage, not a Cargo cache fallback.
//! The existing lock-vendor core decides the closure; the normal native build
//! pipeline alone admits and realizes it. Only live pipeline facts plus signed
//! store verification can construct `VerifiedCargoVendor`.
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use mantle_lock_vendor_core::Limits;
use mantle_lock_vendor_core::plan_cargo_vendor_with_table;
use mantle_lock_vendor_core::shared_table::HashMode;
use mantle_lock_vendor_core::shared_table::SharedHashTable;
use serde::Deserialize;
use serde::Serialize;

use crate::source_bundle::SourceStoreTrust;
use crate::source_bundle::StorePathAttestation;
use crate::source_bundle::verify_signed_store_root;

pub const CARGO_ACQUISITION_SCHEMA: &str = "mantle-cargo-acquisition-v1";
const MAX_RUNTIME_INPUTS: usize = 12;
// Preserve the importer's narrower closure bound; the producer's sharding
// policy and the core's independent limits are unchanged.
const MAX_IMPORT_ARTIFACTS: u32 = 512;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CargoAcquisitionInputs {
    pub bundle: String,
    pub producer: String,
    pub selected_table: String,
    pub python: String,
    pub rust: String,
    pub runtime: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoAcquisitionArtifact {
    pub name: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub hash_mode: String,
    pub git_revision: Option<String>,
    pub reviewed_identity: Option<String>,
    pub vendor_directory: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoAcquisitionInspection {
    pub schema: String,
    pub stage: String,
    pub lock_blake3: String,
    pub selected_table_blake3: String,
    pub selected_table: String,
    pub artifacts: Vec<CargoAcquisitionArtifact>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoAcquisitionBlocker {
    pub stage: String,
    pub code: String,
    pub subject: String,
    pub detail: String,
}

impl std::fmt::Display for CargoAcquisitionBlocker {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(output, "{}: {}: {}: {}", self.stage, self.code, self.subject, self.detail)
    }
}
impl std::error::Error for CargoAcquisitionBlocker {}

type Result<T, E = CargoAcquisitionBlocker> = std::result::Result<T, E>;

fn blocked(stage: &str, code: &str, subject: &str, detail: impl ToString) -> CargoAcquisitionBlocker {
    CargoAcquisitionBlocker {
        stage: stage.into(),
        code: code.into(),
        subject: subject.into(),
        detail: detail.to_string(),
    }
}

/// Inspecting is bounded local input reading only. It neither evaluates a
/// derivation nor downloads anything, and does not confer output authority.
pub fn inspect_cargo_acquisition(lock: &[u8], full_table: &[u8]) -> Result<CargoAcquisitionInspection> {
    let table = SharedHashTable::parse(full_table).map_err(|error| {
        blocked("acquisition-planning", "shared-hash-table", "reviewed-table", format!("{error:?}"))
    })?;
    let limits = Limits {
        artifacts: MAX_IMPORT_ARTIFACTS,
        ..Limits::default()
    };
    let plan = plan_cargo_vendor_with_table(lock, limits, Some(&table)).map_err(|error| {
        blocked(
            "acquisition-planning",
            &format!("{:?}", error.code),
            error.artifact.as_deref().unwrap_or("Cargo.lock"),
            error.detail,
        )
    })?;
    let selected = plan
        .selected_shared_hashes(&table)
        .map_err(|error| {
            blocked(
                "acquisition-planning",
                &format!("{:?}", error.code),
                error.artifact.as_deref().unwrap_or("reviewed-table"),
                error.detail,
            )
        })?
        .render();
    if let Some(artifact) = plan.artifacts.iter().find(|artifact| !artifact.fetch_url.starts_with("https://")) {
        return Err(blocked(
            "acquisition-planning",
            "missing-reviewed-https-transport",
            &artifact.name,
            "native lock-vendor fetches require a reviewed HTTPS transport, not a guessed Git route",
        ));
    }
    let artifacts = plan
        .artifacts
        .into_iter()
        .map(|artifact| CargoAcquisitionArtifact {
            name: artifact.name,
            version: artifact.version,
            url: artifact.fetch_url,
            sha256: artifact.sha256,
            hash_mode: match artifact.hash_mode {
                HashMode::FlatSha256 => "flat-sha256",
                HashMode::RecursiveSha256 => "recursive-sha256",
            }
            .into(),
            git_revision: artifact.git_rev,
            reviewed_identity: artifact.shared_hash_identity,
            vendor_directory: artifact.relative_path,
        })
        .collect();
    Ok(CargoAcquisitionInspection {
        schema: CARGO_ACQUISITION_SCHEMA.into(),
        stage: "acquisition-planning".into(),
        lock_blake3: blake3::hash(lock).to_hex().to_string(),
        selected_table_blake3: blake3::hash(selected.as_bytes()).to_hex().to_string(),
        selected_table: selected,
        artifacts,
        non_claims: vec![
            "not-realized".into(),
            "not-offline-cargo-planning".into(),
            "not-compilation".into(),
        ],
    })
}

pub fn inspect_cargo_workspace(root: &Path, table: &Path) -> Result<CargoAcquisitionInspection> {
    let lock = bounded_read(&root.join("Cargo.lock"), mantle_lock_vendor_core::MAX_LOCK_BYTES as u64)?;
    let table = bounded_read(table, mantle_lock_vendor_core::shared_table::MAX_TABLE_BYTES as u64)?;
    inspect_cargo_acquisition(&lock, &table)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CargoAcquisitionReport {
    pub schema: String,
    pub stage: String,
    pub inspection: CargoAcquisitionInspection,
    pub native_parent_drv_path: String,
    pub canonical_plan_blake3: String,
    pub output: StorePathAttestation,
    pub plan_output: StorePathAttestation,
    pub non_claims: Vec<String>,
}

/// Not deserializable and not constructible by consumers. A checksum receipt
/// or externally supplied JSON build report cannot mint this capability.
pub struct VerifiedCargoVendor {
    logical_root: String,
    physical_root: PathBuf,
    lock_blake3: String,
    attestation: StorePathAttestation,
    state_dir: PathBuf,
    store_dir: String,
    backend: crunch_store::StoreBackend,
    trusted_keys: Vec<nix_compat::narinfo::VerifyingKey>,
}
impl VerifiedCargoVendor {
    pub fn logical_root(&self) -> &str {
        &self.logical_root
    }
    pub fn physical_root(&self) -> &Path {
        &self.physical_root
    }
    pub fn lock_blake3(&self) -> &str {
        &self.lock_blake3
    }
    pub(crate) fn verify_unchanged(&self) -> std::result::Result<(), crate::errors::RunError> {
        let actual =
            verify_signed_store_root(&self.logical_root, &self.state_dir, &self.store_dir, &SourceStoreTrust {
                output_dir: self.physical_root.parent().expect("admitted physical store output has a parent"),
                backend: self.backend,
                trusted_keys: &self.trusted_keys,
            })?;
        if actual != self.attestation {
            return Err(crate::errors::RunError::Internal("acquired vendor signed output identity changed".into()));
        }
        Ok(())
    }
}

fn bounded_read(path: &Path, max: u64) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path)
        .map_err(|error| blocked("acquisition-planning", "input-read", &path.display().to_string(), error))?;
    let mut bytes = Vec::new();
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| blocked("acquisition-planning", "input-read", &path.display().to_string(), error))?;
    if bytes.len() as u64 > max {
        return Err(blocked(
            "acquisition-planning",
            "input-bound",
            &path.display().to_string(),
            "input exceeds existing byte bound",
        ));
    }
    Ok(bytes)
}

fn physical_root(logical: &str, config: &crunch_pipeline::BuildConfig) -> Result<PathBuf> {
    let parsed = nix_compat::store_path::StorePath::<String>::from_absolute_path_with_prefix(
        logical.as_bytes(),
        &config.store_dir,
    )
    .map_err(|error| blocked("acquisition-admission", "declared-store-root", logical, error))?;
    Ok(config.output_dir.join(parsed.to_string()))
}

fn verify_input(logical: &str, config: &crunch_pipeline::BuildConfig) -> Result<StorePathAttestation> {
    // Parsing checks the configured prefix and forbids subpaths before touching
    // physical bytes. Signature verification uses the existing store boundary.
    physical_root(logical, config)?;
    verify_signed_store_root(logical, &config.state_dir, &config.store_dir, &SourceStoreTrust {
        output_dir: &config.output_dir,
        backend: config.backend,
        trusted_keys: &config.trusted_keys,
    })
    .map_err(|error| blocked("acquisition-trust", "signed-output-verification", logical, error))
}

fn validate_inputs(inputs: &CargoAcquisitionInputs, config: &crunch_pipeline::BuildConfig) -> Result<()> {
    if config.trust_unsigned || config.trusted_keys.is_empty() || !config.hermeticity_mode.is_strict() {
        return Err(blocked(
            "acquisition-admission",
            "strict-signed-inputs-required",
            "build-policy",
            "strict hermeticity, explicit trusted keys, and signed output checks are required",
        ));
    }
    if inputs.runtime.len() > MAX_RUNTIME_INPUTS {
        return Err(blocked(
            "acquisition-admission",
            "runtime-bound",
            "runtime",
            "more than 12 declared runtime roots",
        ));
    }
    let mut unique = std::collections::BTreeSet::new();
    for root in [
        &inputs.bundle,
        &inputs.producer,
        &inputs.selected_table,
        &inputs.python,
        &inputs.rust,
    ]
    .into_iter()
    .chain(inputs.runtime.iter())
    {
        if !unique.insert(root) {
            return Err(blocked("acquisition-admission", "duplicate-input", root, "declared roots must be unique"));
        }
        verify_input(root, config)?;
    }
    Ok(())
}

fn quoted(value: &str) -> String {
    // JSON string literals are a subset of Nickel literals except interpolation.
    // Store paths do not contain interpolation; reject it at the API boundary.
    serde_json::to_string(value).expect("serializing a string cannot fail")
}

fn acquisition_recipe(inputs: &CargoAcquisitionInputs) -> String {
    let runtime = inputs.runtime.iter().map(|root| quoted(root)).collect::<Vec<_>>().join(", ");
    format!(
        r#"let mantle = import "lib.ncl" in
let plan = {{
  name = "cargo-import-lock-vendor-plan",
  builder = {producer} ++ "/bin/lock-vendor-producer", system = 'x86_64-linux,
  args = [{bundle} ++ "/workspace/Cargo.lock", {table}, {python} ++ "/bin/python3", {bundle}, {rust}] @ [{runtime}],
  outputs = ["plan"], dynamic_plan_outputs = ["plan"],
  inputs = [{producer}, {bundle}, {table}, {python}, {rust}] @ [{runtime}],
  addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation in
{{
  name = "cargo-import-vendor-acquisition", builder = {python} ++ "/bin/python3", system = 'x86_64-linux,
  args = ["-c", "import os,pathlib; pathlib.Path(os.environ['out']).write_text(os.environ['VENDOR']+'\\n')"],
  env = {{ VENDOR = "{{{{mantle-plan-output:vendor}}}}" }},
  inputs = [{{ name = "vendor", producer = plan, plan_output = "plan", root = "vendor.complete", unit_output = "out" }}, {python}],
  addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation
"#,
        producer = quoted(&inputs.producer),
        bundle = quoted(&inputs.bundle),
        table = quoted(&inputs.selected_table),
        python = quoted(&inputs.python),
        rust = quoted(&inputs.rust)
    )
}

/// Realize declared fetches through the normal scheduler/admission path. No
/// external receipt or JSON report is accepted as native admission evidence.
pub fn realize_cargo_acquisition(
    workspace: &Path,
    inputs: &CargoAcquisitionInputs,
    mut config: crunch_pipeline::BuildConfig,
) -> Result<(VerifiedCargoVendor, CargoAcquisitionReport)> {
    validate_inputs(inputs, &config)?;
    let lock = bounded_read(&workspace.join("Cargo.lock"), mantle_lock_vendor_core::MAX_LOCK_BYTES as u64)?;
    let table = bounded_read(
        &physical_root(&inputs.selected_table, &config)?,
        mantle_lock_vendor_core::shared_table::MAX_TABLE_BYTES as u64,
    )?;
    let inspection = inspect_cargo_acquisition(&lock, &table)?;
    if inspection.artifacts.is_empty() {
        return Err(blocked(
            "acquisition-planning",
            "empty-acquisition",
            "Cargo.lock",
            "use the ordinary local-only import without acquisition",
        ));
    }
    if inspection.selected_table.as_bytes() != table {
        return Err(blocked(
            "acquisition-admission",
            "selected-table-mismatch",
            &inputs.selected_table,
            "producer input must contain only canonical selected reviewed rows",
        ));
    }
    let bundled_lock = bounded_read(
        &physical_root(&inputs.bundle, &config)?.join("workspace/Cargo.lock"),
        mantle_lock_vendor_core::MAX_LOCK_BYTES as u64,
    )?;
    if lock != bundled_lock {
        return Err(blocked(
            "acquisition-admission",
            "lock-source-mismatch",
            "Cargo.lock",
            "workspace lock differs from the signed producer source",
        ));
    }
    let scratch =
        tempfile::tempdir().map_err(|error| blocked("acquisition-realization", "recipe-write", "scratch", error))?;
    config.file = scratch.path().join("acquire.ncl");
    std::fs::write(&config.file, acquisition_recipe(inputs))
        .map_err(|error| blocked("acquisition-realization", "recipe-write", "acquire.ncl", error))?;
    let result = crate::build_cmd::run_build(&config)
        .map_err(|error| blocked("acquisition-realization", "native-build", "vendor.complete", error))?;
    let admitted = admitted_vendor(&result, &config.store_dir, inspection.artifacts.len())?;
    let (parent, digest, logical, plan_logical) = admitted;
    let output = verify_input(&logical, &config)?;
    let plan_output = verify_input(&plan_logical, &config)?;
    verify_emitted_plan(&plan_logical, &digest, &inspection, inputs, &config)?;
    let verified = VerifiedCargoVendor {
        physical_root: physical_root(&logical, &config)?,
        logical_root: logical,
        lock_blake3: inspection.lock_blake3.clone(),
        attestation: output.clone(),
        state_dir: config.state_dir,
        store_dir: config.store_dir,
        backend: config.backend,
        trusted_keys: config.trusted_keys,
    };
    // Independently check Cargo package and file checksums after signed output
    // verification. Neither check is used as a substitute for the other.
    crate::cargo_import::verify_acquired_vendor(workspace, &verified).map_err(|error| {
        blocked("acquisition-vendor-verification", "vendor-checksum", verified.logical_root(), error)
    })?;
    Ok((verified, CargoAcquisitionReport {
        schema: CARGO_ACQUISITION_SCHEMA.into(),
        stage: "acquisition-realized".into(),
        inspection,
        native_parent_drv_path: parent,
        canonical_plan_blake3: digest,
        output,
        plan_output,
        non_claims: vec![
            "not-offline-cargo-planning".into(),
            "not-compilation".into(),
            "not-compiler-correctness".into(),
        ],
    }))
}

fn verify_emitted_plan(
    logical: &str,
    digest: &str,
    inspection: &CargoAcquisitionInspection,
    inputs: &CargoAcquisitionInputs,
    config: &crunch_pipeline::BuildConfig,
) -> Result<()> {
    use crunch_build::dynamic_plan::DynamicBuilder;
    use crunch_build::dynamic_plan::FixedOutputHashAlgo;
    use crunch_build::dynamic_plan::FixedOutputMode;
    let bytes = bounded_read(&physical_root(logical, config)?, crunch_build::dynamic_plan::MAX_DYNAMIC_PLAN_BYTES)?;
    let canonical = crunch_build::dynamic_plan::decode_validated_plan_v1(&bytes, &config.store_dir)
        .map_err(|error| blocked("acquisition-admission", "native-plan-decode", logical, error))?;
    if canonical.digest.as_str() != digest {
        return Err(blocked(
            "acquisition-admission",
            "native-plan-digest-mismatch",
            logical,
            "signed producer bytes differ from accepted native canonical BLAKE3",
        ));
    }
    for (id, path) in [
        ("src.bundle", &inputs.bundle),
        ("src.table", &inputs.selected_table),
        ("src.python", &inputs.python),
        ("src.rust", &inputs.rust),
    ] {
        if !canonical.plan.sources.iter().any(|source| source.id.as_str() == id && source.path.as_str() == path) {
            return Err(blocked(
                "acquisition-admission",
                "native-source-mismatch",
                id,
                "accepted plan source differs from declared signed root",
            ));
        }
    }
    if canonical.plan.sources.len() != 4 + inputs.runtime.len() {
        return Err(blocked(
            "acquisition-admission",
            "native-source-mismatch",
            logical,
            "accepted plan source closure differs from declared roots",
        ));
    }
    for (index, path) in inputs.runtime.iter().enumerate() {
        let id = format!("src.runtime.{index}");
        if !canonical.plan.sources.iter().any(|source| source.id.as_str() == id && source.path.as_str() == path) {
            return Err(blocked(
                "acquisition-admission",
                "native-source-mismatch",
                &id,
                "accepted runtime source differs from declared signed root",
            ));
        }
    }
    let mut fetches = canonical
        .plan
        .units
        .iter()
        .filter(|unit| matches!(&unit.derivation.builder, DynamicBuilder::FetchUrl));
    for artifact in &inspection.artifacts {
        let unit = fetches.next().ok_or_else(|| {
            blocked(
                "acquisition-admission",
                "missing-declared-fetch",
                &artifact.name,
                "no fixed-output fetch for selected lock artifact",
            )
        })?;
        let derivation = &unit.derivation;
        let expected_mode = if artifact.git_revision.is_some() {
            FixedOutputMode::Recursive
        } else {
            FixedOutputMode::Flat
        };
        let fixed_matches = derivation.fixed_output.as_ref().is_some_and(|fixed| {
            fixed.mode == expected_mode && fixed.algo == FixedOutputHashAlgo::Sha256 && fixed.hash == artifact.sha256
        });
        let mut env_matches = derivation.env.get("url") == Some(&artifact.url);
        env_matches &= match &artifact.git_revision {
            Some(revision) => {
                derivation.env.len() == 3
                    && derivation.env.get("type").is_some_and(|kind| kind == "git")
                    && derivation.env.get("rev") == Some(revision)
            }
            None => derivation.env.len() == 1,
        };
        if !fixed_matches
            || !env_matches
            || !derivation.inputs.is_empty()
            || !derivation.args.is_empty()
            || derivation.outputs.len() != 1
            || derivation.outputs[0].as_str() != "out"
        {
            return Err(blocked(
                "acquisition-admission",
                "lock-fetch-mismatch",
                &artifact.name,
                "accepted fixed-output URL/hash/mode/revision differs from selected lock and reviewed table",
            ));
        }
    }
    if fetches.next().is_some() {
        return Err(blocked(
            "acquisition-admission",
            "undeclared-fetch",
            logical,
            "accepted plan contains a fetch outside the selected lock closure",
        ));
    }
    Ok(())
}

fn admitted_vendor(
    result: &crunch_pipeline::PipelineResult,
    prefix: &str,
    artifacts: usize,
) -> Result<(String, String, String, String)> {
    if !result.failed.is_empty() || !result.fod_mismatches.is_empty() || !result.hermeticity_audit_events.is_empty() {
        return Err(blocked(
            "acquisition-realization",
            "native-build-failed",
            "vendor.complete",
            format!(
                "failed={:?}; fixed-output={:?}; audit={:?}",
                result.failed, result.fod_mismatches, result.hermeticity_audit_events
            ),
        ));
    }
    let mut bindings = result
        .plan_output_bindings
        .iter()
        .filter(|row| row.name == "vendor" && row.root == "vendor.complete");
    let binding = bindings.next().ok_or_else(|| {
        blocked(
            "acquisition-admission",
            "missing-native-parent-facts",
            "vendor.complete",
            "no live native vendor output binding",
        )
    })?;
    if bindings.next().is_some() {
        return Err(blocked(
            "acquisition-admission",
            "missing-native-parent-facts",
            "vendor.complete",
            "expected exactly one live native vendor output binding",
        ));
    }
    let parent = binding.producer_drv_path.to_absolute_path_with_prefix(prefix);
    let mut plans = result
        .native_dynamic_plans
        .iter()
        .filter(|row| row.producer_key == parent && row.mode == "native" && row.output_name == "plan");
    let plan = plans.next().ok_or_else(|| {
        blocked("acquisition-admission", "missing-native-parent-facts", &parent, "no actual admitted producer plan")
    })?;
    if plans.next().is_some()
        || binding.status != "bound"
        || binding.failure_reason.is_some()
        || binding.plan_output != "plan"
        || binding.unit_output != "out"
        || binding.root_drv_path.is_none()
    {
        return Err(blocked(
            "acquisition-admission",
            "missing-native-parent-facts",
            &parent,
            "bound root must have its actual admitted producer and root derivation",
        ));
    }
    let digest = plan.canonical_plan_digest.as_ref().filter(|value| {
        value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    if digest.is_none()
        || digest != binding.plan_digest.as_ref()
        || plan.rejection_reason.is_some()
        || plan.scheduler_action != "registered-roots"
        || plan.plan_artifact_path.is_none()
        || plan.accepted_unit_ids.iter().filter(|id| id.starts_with("fetch.")).count() != artifacts
        || !plan.accepted_unit_ids.iter().any(|id| id == "vendor.complete")
    {
        return Err(blocked(
            "acquisition-admission",
            "native-plan-mismatch",
            &parent,
            "accepted plan digest, fetched closure, and bound vendor root must agree",
        ));
    }
    let output = binding.output_path.as_ref().ok_or_else(|| {
        blocked(
            "acquisition-admission",
            "missing-vendor-output",
            "vendor.complete",
            "native binding has no realized output",
        )
    })?;
    let plan_path = plan.plan_artifact_path.as_ref().expect("checked above").to_absolute_path_with_prefix(prefix);
    Ok((
        parent,
        digest.expect("checked above").clone(),
        output.to_absolute_path_with_prefix(prefix),
        plan_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    const LOCK: &[u8] = include_bytes!("../fixtures/lock-vendor-two-crate/Cargo.lock");
    const TABLE: &[u8] = b"mantle-shared-lock-hashes-v1\n";

    #[test]
    fn unrelated_lock_and_table_rows_preserve_existing_artifacts() {
        let original = inspect_cargo_acquisition(LOCK, TABLE).unwrap();
        let mut extended = LOCK.to_vec();
        extended.extend_from_slice(b"\n[[package]]\nname = \"z-extra\"\nversion = \"1.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n");
        let table = b"mantle-shared-lock-hashes-v1\ncargo/unused@1.0.0/git+https://example.org/repo#1111111111111111111111111111111111111111\trecursive-sha256\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n";
        let changed = inspect_cargo_acquisition(&extended, table).unwrap();
        assert_eq!(changed.selected_table_blake3, original.selected_table_blake3);
        for artifact in &original.artifacts {
            assert_eq!(changed.artifacts.iter().find(|row| row.name == artifact.name), Some(artifact));
        }
    }

    #[test]
    fn changed_dependency_only_changes_its_acquisition_record() {
        let original = inspect_cargo_acquisition(LOCK, TABLE).unwrap();
        let lock = std::str::from_utf8(LOCK).unwrap().replace(&original.artifacts[0].sha256, &"b".repeat(64));
        let changed = inspect_cargo_acquisition(lock.as_bytes(), TABLE).unwrap();
        assert_ne!(original.artifacts[0].sha256, changed.artifacts[0].sha256);
        assert_eq!(original.artifacts[1], changed.artifacts[1]);
    }

    #[test]
    fn missing_lock_hash_and_unreviewed_git_are_named_at_planning_boundary() {
        let lock = std::str::from_utf8(LOCK).unwrap();
        let missing = lock.lines().filter(|line| !line.starts_with("checksum =")).collect::<Vec<_>>().join("\n");
        let error = inspect_cargo_acquisition(missing.as_bytes(), TABLE).unwrap_err();
        assert_eq!(error.subject, "crc64");
        assert_eq!(error.stage, "acquisition-planning");
        let git = b"version = 4\n[[package]]\nname = \"unreviewed\"\nversion = \"1.0.0\"\nsource = \"git+https://example.org/repo#1111111111111111111111111111111111111111\"\n";
        assert_eq!(inspect_cargo_acquisition(git, TABLE).unwrap_err().subject, "unreviewed");
    }

    #[test]
    fn absent_live_native_parent_facts_cannot_mint_a_verified_vendor() {
        let result = crunch_pipeline::PipelineResult {
            outcomes: Vec::new(),
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: std::collections::HashMap::new(),
            hermeticity_mode: crunch_build::HermeticityMode::Strict,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            finish_gate_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            plan_output_bindings: Vec::new(),
            priority_decisions: Vec::new(),
            overlay_report: None,
            store_layer_selections: Vec::new(),
        };
        let denial = admitted_vendor(&result, "/nix/store", 2).unwrap_err();
        assert_eq!(denial.code, "missing-native-parent-facts");
        assert_eq!(denial.stage, "acquisition-admission");
    }
}
