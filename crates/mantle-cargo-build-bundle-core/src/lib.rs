#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::ambiguous_params)]

//! Cargo handoff identity and compatibility, without source, plan, or trust authority.
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::NativeInputDeclaration;
use mantle_rust_plan_core::ResolvedNativeInputs;
use mantle_rust_plan_core::resolve_native_inputs;
use serde::Deserialize;
use serde::Serialize;
pub mod application;

pub const SCHEMA: &str = "mantle-cargo-build-bundle-v1";
pub const MAX_MANIFEST_BYTES: usize = 8 << 20;
pub const MAX_ENTRIES: usize = 4096;
pub const MAX_TEXT_BYTES: usize = 16_384;
pub const NON_CLAIM: &str = "exported Cargo graph replay; source availability, plan admission, trusted cached results and execution are separate; no Cargo-free, compiler, reproducibility or release claim";

/// Validation and serialization failures of the Cargo handoff contract.
#[derive(Debug)]
pub enum BundleError {
    CompatibilityMismatch,
    NoncanonicalOrTampered,
    ManifestLimit,
    SchemaUnsupported,
    PlanningVersionUnsupported,
    ProducerRevisionInvalid,
    ArtifactMissing { name: &'static str },
    ArtifactFileDuplicate,
    OracleProvenanceMissing { name: &'static str },
    OracleInputIdentityMismatch,
    TargetUnsupported,
    PrefixUnsupported,
    PolicyUnsupported,
    MandatorySchemaUnsupported,
    AdmissionLimitsUnsupported,
    EntryLimit,
    RequiredInputMissing,
    TextLimit,
    NestingLimit,
    ToolMissing { role: &'static str },
    ProducerOutputMissing { name: &'static str },
    UnitCycle,
    RootsMissing,
    RootMissing,
    DependencyLimit,
    DependencyMissing,
    PackageSourceMissing,
    AuxiliaryUnitInvalid,
    NativeSchemaUnsupported,
    NativePackageMissing,
    NativeResolutionMismatch,
    PayloadUndeclared { path: String },
    StorePathInvalid,
    ArtifactPathInvalid,
    DigestInvalid,
    NativeResolution(mantle_rust_plan_core::NativeInputError),
    Serialization(serde_json::Error),
}

impl core::fmt::Display for BundleError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CompatibilityMismatch => formatter.write_str("cargo-bundle-compatibility-mismatch"),
            Self::NoncanonicalOrTampered => formatter.write_str("cargo-bundle-noncanonical-or-tampered"),
            Self::ManifestLimit => formatter.write_str("cargo-bundle-manifest-limit"),
            Self::SchemaUnsupported => formatter.write_str("cargo-bundle-schema-unsupported"),
            Self::PlanningVersionUnsupported => formatter.write_str("cargo-bundle-planning-version-unsupported"),
            Self::ProducerRevisionInvalid => formatter.write_str("cargo-bundle-producer-revision-invalid"),
            Self::ArtifactMissing { name } => write!(formatter, "cargo-bundle-artifact-missing:{name}"),
            Self::ArtifactFileDuplicate => formatter.write_str("cargo-bundle-artifact-file-duplicate"),
            Self::OracleProvenanceMissing { name } => {
                write!(formatter, "cargo-bundle-oracle-provenance-missing:{name}")
            }
            Self::OracleInputIdentityMismatch => formatter.write_str("cargo-bundle-oracle-input-identity-mismatch"),
            Self::TargetUnsupported => formatter.write_str("cargo-bundle-target-unsupported"),
            Self::PrefixUnsupported => formatter.write_str("cargo-bundle-prefix-unsupported"),
            Self::PolicyUnsupported => formatter.write_str("cargo-bundle-policy-unsupported"),
            Self::MandatorySchemaUnsupported => formatter.write_str("cargo-bundle-mandatory-schema-unsupported"),
            Self::AdmissionLimitsUnsupported => formatter.write_str("cargo-bundle-admission-limits-unsupported"),
            Self::EntryLimit => formatter.write_str("cargo-bundle-entry-limit"),
            Self::RequiredInputMissing => formatter.write_str("cargo-bundle-required-input-missing"),
            Self::TextLimit => formatter.write_str("cargo-bundle-text-limit"),
            Self::NestingLimit => formatter.write_str("cargo-bundle-nesting-limit"),
            Self::ToolMissing { role } => write!(formatter, "cargo-bundle-tool-missing:{role}"),
            Self::ProducerOutputMissing { name } => write!(formatter, "cargo-bundle-producer-output-missing:{name}"),
            Self::UnitCycle => formatter.write_str("cargo-bundle-unit-cycle"),
            Self::RootsMissing => formatter.write_str("cargo-bundle-roots-missing"),
            Self::RootMissing => formatter.write_str("cargo-bundle-root-missing"),
            Self::DependencyLimit => formatter.write_str("cargo-bundle-dependency-limit"),
            Self::DependencyMissing => formatter.write_str("cargo-bundle-dependency-missing"),
            Self::PackageSourceMissing => formatter.write_str("cargo-bundle-package-source-missing"),
            Self::AuxiliaryUnitInvalid => formatter.write_str("cargo-bundle-auxiliary-unit-invalid"),
            Self::NativeSchemaUnsupported => formatter.write_str("cargo-bundle-native-schema-unsupported"),
            Self::NativePackageMissing => formatter.write_str("cargo-bundle-native-package-missing"),
            Self::NativeResolutionMismatch => formatter.write_str("cargo-bundle-native-resolution-mismatch"),
            Self::PayloadUndeclared { path } => write!(formatter, "cargo-bundle-payload-undeclared:{path}"),
            Self::StorePathInvalid => formatter.write_str("cargo-bundle-store-path-invalid"),
            Self::ArtifactPathInvalid => formatter.write_str("cargo-bundle-artifact-path-invalid"),
            Self::DigestInvalid => formatter.write_str("cargo-bundle-digest-invalid"),
            Self::NativeResolution(error) => core::fmt::Display::fmt(error, formatter),
            Self::Serialization(error) => core::fmt::Display::fmt(error, formatter),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    /// Relative transport name, never a producer filesystem path.
    pub file: String,
    pub blake3: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePackage {
    pub schema_version: u32,
    pub declaration: NativeInputDeclaration,
    pub resolved: ResolvedNativeInputs,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    /// None only for the generic target-facts helper unit, checked against native plan args.
    pub package_id: Option<String>,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    pub system: String,
    pub host_triple: String,
    pub target_triple: String,
    pub store_prefix: String,
    pub sandbox: String,
    pub network: String,
    pub policy_blake3: String,
    pub trust_policy_blake3: String,
    pub plan_schema: String,
    pub source_schema: String,
    pub receipt_schema: String,
    pub max_plan_bytes: u64,
    pub max_units: u32,
    pub max_sources: u32,
    pub max_direct_inputs: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub cargo_unit_graph_version: u32,
    pub planner_receipt_schema_version: u32,
    pub producer_revision: String,
    pub oracle_inputs: BTreeMap<String, String>,
    pub evidence: BTreeMap<String, Artifact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub version: u32,
    pub compatibility: Compatibility,
    pub profile: String,
    pub selected_features: Vec<String>,
    pub no_default_features: bool,
    pub declared_env: BTreeMap<String, String>,
    /// Exact executable paths (including bin suffix) for the seven required roles.
    pub tools: BTreeMap<String, String>,
    /// Every required signed store root, including cached producer outputs.
    pub required_store_paths: Vec<String>,
    pub producer_outputs: BTreeMap<String, String>,
    pub source_manifest_blake3: String,
    pub receipt_bundle_blake3: String,
    pub lock_blake3: String,
    pub canonical_plan_blake3: String,
    /// Exact Cargo package identity -> immutable source-bundle record identity.
    pub package_sources: BTreeMap<String, String>,
    /// Registry/Git coordinates -> verified immutable source record identity.
    pub registry_git_sources: BTreeMap<String, String>,
    pub vendor_store_path: Option<String>,
    pub units: BTreeMap<String, Unit>,
    pub roots: Vec<String>,
    pub native_inputs: BTreeMap<String, NativePackage>,
    pub provenance: Provenance,
    pub artifacts: BTreeMap<String, Artifact>,
    pub bundle_blake3: String,
    pub non_claim: String,
}

pub fn digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn canonicalize(mut manifest: Manifest) -> Result<Manifest, BundleError> {
    manifest.selected_features.sort();
    manifest.selected_features.dedup();
    manifest.required_store_paths.sort();
    manifest.required_store_paths.dedup();
    manifest.roots.sort();
    manifest.roots.dedup();
    for unit in manifest.units.values_mut() {
        unit.dependencies.sort();
        unit.dependencies.dedup();
        unit.features.sort();
        unit.features.dedup();
    }
    manifest.bundle_blake3.clear();
    validate_shape(&manifest)?;
    manifest.bundle_blake3 = digest(&encode(&manifest)?);
    Ok(manifest)
}

pub fn verify(manifest: &Manifest, expected: &Compatibility) -> Result<(), BundleError> {
    validate_shape(manifest)?;
    if &manifest.compatibility != expected {
        return Err(BundleError::CompatibilityMismatch);
    }
    let canonical = canonicalize(manifest.clone())?;
    if &canonical != manifest {
        return Err(BundleError::NoncanonicalOrTampered);
    }
    Ok(())
}

pub fn encode(manifest: &Manifest) -> Result<Vec<u8>, BundleError> {
    let bytes = serde_json::to_vec(manifest).map_err(BundleError::Serialization)?;
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(BundleError::ManifestLimit);
    }
    Ok(bytes)
}

fn validate_shape(m: &Manifest) -> Result<(), BundleError> {
    validate_manifest_schema(m)?;
    validate_compatibility(&m.compatibility)?;
    validate_planning_versions(m)?;
    validate_producer_revision(m)?;
    validate_manifest_digests(m)?;
    validate_collections(m)?;
    validate_tools(m)?;
    validate_graph(m)?;
    validate_native(m)?;
    validate_required_artifacts(m)?;
    validate_artifact_files(m)?;
    validate_oracle_evidence(m)?;
    validate_oracle_digests(m)?;
    debug_assert!(m.artifacts.contains_key("source"));
    debug_assert!(m.provenance.evidence.contains_key("cargo-metadata"));
    Ok(())
}

/// The manifest declares exactly the supported schema, version and non-claim.
fn validate_manifest_schema(m: &Manifest) -> Result<(), BundleError> {
    if m.schema != SCHEMA || m.version != 1 || m.non_claim != NON_CLAIM {
        return Err(BundleError::SchemaUnsupported);
    }
    debug_assert_eq!(m.schema, SCHEMA);
    debug_assert_eq!(m.version, 1);
    Ok(())
}

/// The planner must have produced the supported graph and receipt versions.
fn validate_planning_versions(m: &Manifest) -> Result<(), BundleError> {
    let provenance = &m.provenance;
    if provenance.cargo_unit_graph_version != 1 || provenance.planner_receipt_schema_version != 1 {
        return Err(BundleError::PlanningVersionUnsupported);
    }
    debug_assert_eq!(provenance.cargo_unit_graph_version, 1);
    debug_assert_eq!(provenance.planner_receipt_schema_version, 1);
    Ok(())
}

/// The producer revision must be a full hexadecimal commit identity.
fn validate_producer_revision(m: &Manifest) -> Result<(), BundleError> {
    if !is_hex(&m.provenance.producer_revision, 40) {
        return Err(BundleError::ProducerRevisionInvalid);
    }
    debug_assert!(is_hex(&m.provenance.producer_revision, 40));
    Ok(())
}

/// The four manifest-level digests must be well formed.
fn validate_manifest_digests(m: &Manifest) -> Result<(), BundleError> {
    for value in [
        &m.source_manifest_blake3,
        &m.receipt_bundle_blake3,
        &m.lock_blake3,
        &m.canonical_plan_blake3,
    ] {
        require_digest(value)?;
    }
    debug_assert!(is_hex(&m.source_manifest_blake3, 64));
    debug_assert!(is_hex(&m.canonical_plan_blake3, 64));
    Ok(())
}

/// Every role artifact must be present in the manifest.
fn validate_required_artifacts(m: &Manifest) -> Result<(), BundleError> {
    for name in ["source", "receipts", "plan", "goals", "store", "lock", "config"] {
        if !m.artifacts.contains_key(name) {
            return Err(BundleError::ArtifactMissing { name });
        }
    }
    debug_assert!(m.artifacts.contains_key("source"));
    debug_assert!(m.artifacts.contains_key("config"));
    Ok(())
}

/// Artifact transport names are unique across artifacts and provenance evidence.
fn validate_artifact_files(m: &Manifest) -> Result<(), BundleError> {
    let mut files = BTreeSet::new();
    for artifact in m.artifacts.values().chain(m.provenance.evidence.values()) {
        validate_artifact(artifact)?;
        if !files.insert(&artifact.file) {
            return Err(BundleError::ArtifactFileDuplicate);
        }
    }
    debug_assert!(!files.is_empty());
    debug_assert!(files.len() <= m.artifacts.len().saturating_add(m.provenance.evidence.len()));
    Ok(())
}

/// Each oracle input must name the exact evidence artifact it was derived from.
fn validate_oracle_evidence(m: &Manifest) -> Result<(), BundleError> {
    for name in [
        "cargo-metadata",
        "cargo-unit-graph",
        "rust-plan-receipt",
        "cargo-unit-plan-evidence",
        "cargo-unit-bindings",
    ] {
        let artifact = m.provenance.evidence.get(name).ok_or(BundleError::OracleProvenanceMissing { name })?;
        if m.provenance.oracle_inputs.get(name) != Some(&artifact.blake3) {
            return Err(BundleError::OracleInputIdentityMismatch);
        }
    }
    debug_assert!(m.provenance.evidence.contains_key("cargo-unit-graph"));
    debug_assert!(m.provenance.oracle_inputs.contains_key("cargo-metadata"));
    Ok(())
}

/// Every recorded oracle input identity must be a well-formed digest.
fn validate_oracle_digests(m: &Manifest) -> Result<(), BundleError> {
    for value in m.provenance.oracle_inputs.values() {
        require_digest(value)?;
    }
    debug_assert!(m.provenance.oracle_inputs.values().all(|value| is_hex(value, 64)));
    Ok(())
}

fn validate_compatibility(c: &Compatibility) -> Result<(), BundleError> {
    validate_target_identity(c)?;
    validate_store_prefix(c)?;
    validate_sandbox_policy(c)?;
    validate_mandatory_schemas(c)?;
    validate_admission_limits(c)?;
    require_digest(&c.policy_blake3)?;
    require_digest(&c.trust_policy_blake3)
}

/// The bundle only describes the admitted native x86_64 target.
fn validate_target_identity(c: &Compatibility) -> Result<(), BundleError> {
    let is_admitted_system = c.system == "x86_64-linux";
    let is_admitted_host_triple = c.host_triple == "x86_64-unknown-linux-gnu";
    let is_matching_target_triple = c.target_triple == c.host_triple;
    if !is_admitted_system || !is_admitted_host_triple || !is_matching_target_triple {
        return Err(BundleError::TargetUnsupported);
    }
    debug_assert_eq!(c.target_triple, c.host_triple);
    Ok(())
}

/// The store prefix must be one of the two admitted roots.
fn validate_store_prefix(c: &Compatibility) -> Result<(), BundleError> {
    if c.store_prefix != "/nix/store" && c.store_prefix != "/mantle/store" {
        return Err(BundleError::PrefixUnsupported);
    }
    debug_assert!(c.store_prefix == "/nix/store" || c.store_prefix == "/mantle/store");
    Ok(())
}

/// The admitted policy is a native sandbox with no network.
fn validate_sandbox_policy(c: &Compatibility) -> Result<(), BundleError> {
    if c.sandbox != "native" || c.network != "denied" {
        return Err(BundleError::PolicyUnsupported);
    }
    debug_assert_eq!(c.network, "denied");
    Ok(())
}

/// The mandatory plan, source and receipt schemas are fixed.
fn validate_mandatory_schemas(c: &Compatibility) -> Result<(), BundleError> {
    if c.plan_schema != "mantle-plan-v2"
        || c.source_schema != "mantle-source-bundle-v1"
        || c.receipt_schema != "mantle-build-receipt-bundle-v1"
    {
        return Err(BundleError::MandatorySchemaUnsupported);
    }
    debug_assert_eq!(c.plan_schema, "mantle-plan-v2");
    Ok(())
}

/// The admission limits are fixed by the build contract.
fn validate_admission_limits(c: &Compatibility) -> Result<(), BundleError> {
    let is_admitted_plan_budget = c.max_plan_bytes == 4_194_304;
    let is_admitted_unit_budget = c.max_units == 4096;
    if !is_admitted_plan_budget || !is_admitted_unit_budget {
        return Err(BundleError::AdmissionLimitsUnsupported);
    }
    let is_admitted_source_budget = c.max_sources == 256;
    let is_admitted_direct_input_budget = c.max_direct_inputs == 256;
    if !is_admitted_source_budget || !is_admitted_direct_input_budget {
        return Err(BundleError::AdmissionLimitsUnsupported);
    }
    debug_assert_eq!(c.max_plan_bytes, 4_194_304);
    debug_assert_eq!(c.max_units, 4096);
    Ok(())
}

fn validate_collections(m: &Manifest) -> Result<(), BundleError> {
    validate_collection_sizes(m)?;
    validate_required_inputs(m)?;
    // This bounds every string, including nested declarations, before downstream effects.
    let value = serde_json::to_value(m).map_err(BundleError::Serialization)?;
    validate_value(&value, 0)
}

/// Every manifest collection is bounded by the entry limit.
fn validate_collection_sizes(m: &Manifest) -> Result<(), BundleError> {
    for count in [
        m.tools.len(),
        m.required_store_paths.len(),
        m.units.len(),
        m.roots.len(),
        m.native_inputs.len(),
        m.artifacts.len(),
        m.package_sources.len(),
        m.registry_git_sources.len(),
        m.provenance.evidence.len(),
        m.provenance.oracle_inputs.len(),
        m.declared_env.len(),
        m.selected_features.len(),
    ] {
        if count > MAX_ENTRIES {
            return Err(BundleError::EntryLimit);
        }
    }
    debug_assert!(m.units.len() <= MAX_ENTRIES);
    debug_assert!(m.artifacts.len() <= MAX_ENTRIES);
    Ok(())
}

/// The profile, the unit set and the package source set must all be present.
fn validate_required_inputs(m: &Manifest) -> Result<(), BundleError> {
    let is_profile_present = !m.profile.is_empty();
    let is_profile_admissible = m.profile.len() <= MAX_TEXT_BYTES;
    if !is_profile_present || !is_profile_admissible {
        return Err(BundleError::RequiredInputMissing);
    }
    if m.units.is_empty() || m.package_sources.is_empty() {
        return Err(BundleError::RequiredInputMissing);
    }
    debug_assert!(!m.profile.is_empty());
    debug_assert!(!m.units.is_empty());
    Ok(())
}

/// Maximum admitted JSON nesting depth of a manifest value.
const MAX_VALUE_DEPTH: usize = 16;

/// One pending step of the iterative manifest-value walk.
enum ValueStep<'a> {
    /// Validate this value at this depth.
    Value(&'a serde_json::Value, usize),
    /// Validate this object key before descending into its value.
    Key(&'a str),
}

/// Walk the manifest value with an explicit stack, so nesting depth stays bounded
/// and the walk never re-enters itself.
fn validate_value(value: &serde_json::Value, depth: usize) -> Result<(), BundleError> {
    let mut pending = Vec::with_capacity(MAX_VALUE_DEPTH);
    pending.push(ValueStep::Value(value, depth));
    while let Some(step) = pending.pop() {
        apply_value_step(step, &mut pending)?;
    }
    debug_assert!(pending.is_empty());
    Ok(())
}

fn apply_value_step<'a>(step: ValueStep<'a>, pending: &mut Vec<ValueStep<'a>>) -> Result<(), BundleError> {
    match step {
        ValueStep::Key(key) => {
            if key.len() > MAX_TEXT_BYTES {
                return Err(BundleError::TextLimit);
            }
        }
        ValueStep::Value(current, current_depth) => {
            if current_depth > MAX_VALUE_DEPTH {
                return Err(BundleError::NestingLimit);
            }
            push_value_steps(current, current_depth, pending)?;
        }
    }
    Ok(())
}

/// Push the children of one value, deepest-last, preserving the depth-first order
/// in which the recursive walk used to visit them.
fn push_value_steps<'a>(
    value: &'a serde_json::Value,
    depth: usize,
    pending: &mut Vec<ValueStep<'a>>,
) -> Result<(), BundleError> {
    match value {
        serde_json::Value::String(text) if text.len() > MAX_TEXT_BYTES || text.contains('\0') => {
            Err(BundleError::TextLimit)
        }
        serde_json::Value::Array(items) => {
            if items.len() > MAX_ENTRIES {
                return Err(BundleError::EntryLimit);
            }
            let child_depth = depth.saturating_add(1);
            for item in items.iter().rev() {
                pending.push(ValueStep::Value(item, child_depth));
            }
            debug_assert!(items.len() <= MAX_ENTRIES);
            debug_assert!(child_depth > depth);
            Ok(())
        }
        serde_json::Value::Object(items) => {
            if items.len() > MAX_ENTRIES {
                return Err(BundleError::EntryLimit);
            }
            let child_depth = depth.saturating_add(1);
            for (key, item) in items.iter().rev() {
                pending.push(ValueStep::Value(item, child_depth));
                pending.push(ValueStep::Key(key.as_str()));
            }
            debug_assert!(items.len() <= MAX_ENTRIES);
            debug_assert!(child_depth > depth);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_tools(m: &Manifest) -> Result<(), BundleError> {
    for role in ["rustc", "cargo", "toolchain", "producer", "planner", "helper", "linker"] {
        let path = m.tools.get(role).ok_or(BundleError::ToolMissing { role })?;
        require_covered(path, m)?;
    }
    for path in &m.required_store_paths {
        store_root(path, &m.compatibility.store_prefix)?;
    }
    for name in ["out", "plan", "sources"] {
        let path = m.producer_outputs.get(name).ok_or(BundleError::ProducerOutputMissing { name })?;
        require_covered(path, m)?;
    }
    if let Some(vendor) = &m.vendor_store_path {
        require_covered(vendor, m)?;
    }
    Ok(())
}

/// The Cargo unit graph, indexed in `m.units` iteration order.
struct UnitGraph {
    /// For every unit, the units that depend on it.
    dependents: Vec<Vec<usize>>,
    /// For every unit, the number of dependency edges that are not yet satisfied.
    indegree: Vec<usize>,
}

fn validate_graph(m: &Manifest) -> Result<(), BundleError> {
    validate_graph_roots(m)?;
    let mut graph = build_unit_graph(m)?;
    let admitted = admit_graph_units(&graph.dependents, &mut graph.indegree);
    if admitted != m.units.len() {
        return Err(BundleError::UnitCycle);
    }
    debug_assert!(admitted <= m.units.len());
    debug_assert_eq!(graph.dependents.len(), m.units.len());
    Ok(())
}

/// Every declared root must name a unit of the graph.
fn validate_graph_roots(m: &Manifest) -> Result<(), BundleError> {
    if m.roots.is_empty() {
        return Err(BundleError::RootsMissing);
    }
    for root in &m.roots {
        if !m.units.contains_key(root) {
            return Err(BundleError::RootMissing);
        }
    }
    debug_assert!(!m.roots.is_empty());
    debug_assert!(m.units.contains_key(m.roots[0].as_str()));
    Ok(())
}

/// Build the unit adjacency, rejecting unknown dependencies and over-wide edges.
fn build_unit_graph(m: &Manifest) -> Result<UnitGraph, BundleError> {
    let ids: Vec<&str> = m.units.keys().map(|id| id.as_str()).collect();
    let index_of: BTreeMap<&str, usize> = ids.iter().enumerate().map(|(index, id)| (*id, index)).collect();
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); ids.len()];
    let mut indegree: Vec<usize> = vec![0; ids.len()];
    for (index, unit) in m.units.values().enumerate() {
        validate_unit_identity(m, unit)?;
        if unit.dependencies.len() > 256 {
            return Err(BundleError::DependencyLimit);
        }
        indegree[index] = unit.dependencies.len();
        for dependency in &unit.dependencies {
            let dependency_index = *index_of.get(dependency.as_str()).ok_or(BundleError::DependencyMissing)?;
            dependents[dependency_index].push(index);
        }
    }
    debug_assert_eq!(dependents.len(), m.units.len());
    debug_assert_eq!(indegree.len(), m.units.len());
    Ok(UnitGraph { dependents, indegree })
}

/// A unit either names an admitted package source or carries no facts at all.
fn validate_unit_identity(m: &Manifest, unit: &Unit) -> Result<(), BundleError> {
    if let Some(package) = &unit.package_id {
        if !m.package_sources.contains_key(package) {
            return Err(BundleError::PackageSourceMissing);
        }
    } else if !unit.features.is_empty() || !unit.dependencies.is_empty() {
        return Err(BundleError::AuxiliaryUnitInvalid);
    }
    debug_assert!(unit.package_id.is_some() || unit.dependencies.is_empty());
    Ok(())
}

/// Admit every unit whose dependencies are already satisfied; a cycle admits fewer.
fn admit_graph_units(dependents: &[Vec<usize>], indegree: &mut [usize]) -> usize {
    let mut ready: Vec<usize> = Vec::with_capacity(indegree.len());
    ready.extend(indegree.iter().enumerate().filter_map(|(index, degree)| (*degree == 0).then_some(index)));
    debug_assert!(ready.len() <= indegree.len());
    let mut admitted = 0usize;
    while let Some(index) = ready.pop() {
        admitted = admitted.saturating_add(1);
        admit_graph_consumer(index, dependents, indegree, &mut ready);
    }
    debug_assert!(admitted <= indegree.len());
    admitted
}

/// Release one admitted unit and enqueue the consumers it unblocks.
fn admit_graph_consumer(index: usize, dependents: &[Vec<usize>], indegree: &mut [usize], ready: &mut Vec<usize>) {
    for consumer in &dependents[index] {
        let degree = indegree[*consumer];
        let remaining = degree.saturating_sub(1);
        indegree[*consumer] = remaining;
        if remaining == 0 {
            ready.push(*consumer);
        }
    }
}

fn validate_native(m: &Manifest) -> Result<(), BundleError> {
    for (package, native) in &m.native_inputs {
        if native.schema_version != 1 {
            return Err(BundleError::NativeSchemaUnsupported);
        }
        if !m.package_sources.contains_key(package) {
            return Err(BundleError::NativePackageMissing);
        }
        let resolved = resolve_native_inputs(&native.declaration).map_err(BundleError::NativeResolution)?;
        if resolved != native.resolved {
            return Err(BundleError::NativeResolutionMismatch);
        }
        for path in &resolved.inputs {
            require_covered(path, m)?;
        }
    }
    Ok(())
}

fn require_covered(path: &str, m: &Manifest) -> Result<(), BundleError> {
    let root = store_root(path, &m.compatibility.store_prefix)?;
    if !m.required_store_paths.iter().any(|candidate| candidate == root) {
        return Err(BundleError::PayloadUndeclared { path: path.to_string() });
    }
    Ok(())
}

pub fn store_root<'a>(path: &'a str, prefix: &str) -> Result<&'a str, BundleError> {
    let tail = path
        .strip_prefix(prefix)
        .and_then(|tail| tail.strip_prefix('/'))
        .ok_or(BundleError::StorePathInvalid)?;
    let root = tail.split('/').next().unwrap_or("");
    let (hash, name) = root.split_once('-').ok_or(BundleError::StorePathInvalid)?;
    let is_hash_valid =
        hash.len() == 32 && hash.bytes().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&byte));
    let is_name_present = !name.is_empty();
    let has_traversal_part = path.split('/').any(|part| part == "." || part == "..");
    let is_valid_root = is_hash_valid && is_name_present && !has_traversal_part;
    if !is_valid_root {
        return Err(BundleError::StorePathInvalid);
    }
    let root_end = prefix.len().saturating_add(1).saturating_add(root.len());
    debug_assert!(root_end <= path.len());
    Ok(&path[..root_end])
}
fn validate_artifact(a: &Artifact) -> Result<(), BundleError> {
    let is_name_present = !a.file.is_empty();
    let is_relative_name = !a.file.starts_with('/');
    let is_plain_name = !a.file.contains('\\');
    let has_bad_segment = a.file.split('/').any(|p| p.is_empty() || p == "." || p == "..");
    let is_well_formed_name = is_name_present && is_relative_name && is_plain_name;
    let is_empty_payload = a.bytes == 0;
    if !is_well_formed_name || has_bad_segment || is_empty_payload {
        return Err(BundleError::ArtifactPathInvalid);
    }
    require_digest(&a.blake3)
}
fn require_digest(value: &str) -> Result<(), BundleError> {
    if !is_hex(value, 64) {
        return Err(BundleError::DigestInvalid);
    }
    Ok(())
}
fn is_hex(value: &str, bytes: usize) -> bool {
    value.len() == bytes && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
