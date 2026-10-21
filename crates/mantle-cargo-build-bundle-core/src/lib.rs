#![no_std]

//! Cargo handoff identity and compatibility, without source, plan, or trust authority.
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use mantle_rust_plan_core::NativeInputDeclaration;
use mantle_rust_plan_core::ResolvedNativeInputs;
use mantle_rust_plan_core::resolve_native_inputs;
use serde::Deserialize;
use serde::Serialize;
pub mod application;

pub const SCHEMA: &str = "mantle-cargo-build-bundle-v1";
pub const MAX_MANIFEST_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 4096;
pub const MAX_TEXT_BYTES: usize = 16_384;
pub const NON_CLAIM: &str = "exported Cargo graph replay; source availability, plan admission, trusted cached results and execution are separate; no Cargo-free, compiler, reproducibility or release claim";

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

pub fn canonicalize(mut manifest: Manifest) -> Result<Manifest, String> {
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

pub fn verify(manifest: &Manifest, expected: &Compatibility) -> Result<(), String> {
    validate_shape(manifest)?;
    if &manifest.compatibility != expected {
        return Err("cargo-bundle-compatibility-mismatch".to_string());
    }
    let canonical = canonicalize(manifest.clone())?;
    if &canonical != manifest {
        return Err("cargo-bundle-noncanonical-or-tampered".to_string());
    }
    Ok(())
}

pub fn encode(manifest: &Manifest) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec(manifest).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err("cargo-bundle-manifest-limit".to_string());
    }
    Ok(bytes)
}

fn validate_shape(m: &Manifest) -> Result<(), String> {
    if m.schema != SCHEMA || m.version != 1 || m.non_claim != NON_CLAIM {
        return Err("cargo-bundle-schema-unsupported".to_string());
    }
    validate_compatibility(&m.compatibility)?;
    if m.provenance.cargo_unit_graph_version != 1 || m.provenance.planner_receipt_schema_version != 1 {
        return Err("cargo-bundle-planning-version-unsupported".to_string());
    }
    if !is_hex(&m.provenance.producer_revision, 40) {
        return Err("cargo-bundle-producer-revision-invalid".to_string());
    }
    for value in [
        &m.source_manifest_blake3,
        &m.receipt_bundle_blake3,
        &m.lock_blake3,
        &m.canonical_plan_blake3,
    ] {
        require_digest(value)?;
    }
    validate_collections(m)?;
    validate_tools(m)?;
    validate_graph(m)?;
    validate_native(m)?;
    for name in ["source", "receipts", "plan", "goals", "store", "lock", "config"] {
        if !m.artifacts.contains_key(name) {
            return Err(format!("cargo-bundle-artifact-missing:{name}"));
        }
    }
    let mut files = BTreeSet::new();
    for artifact in m.artifacts.values().chain(m.provenance.evidence.values()) {
        validate_artifact(artifact)?;
        if !files.insert(&artifact.file) {
            return Err("cargo-bundle-artifact-file-duplicate".to_string());
        }
    }
    for name in [
        "cargo-metadata",
        "cargo-unit-graph",
        "rust-plan-receipt",
        "cargo-unit-plan-evidence",
        "cargo-unit-bindings",
    ] {
        let artifact = m
            .provenance
            .evidence
            .get(name)
            .ok_or_else(|| format!("cargo-bundle-oracle-provenance-missing:{name}"))?;
        if m.provenance.oracle_inputs.get(name) != Some(&artifact.blake3) {
            return Err("cargo-bundle-oracle-input-identity-mismatch".to_string());
        }
    }
    for value in m.provenance.oracle_inputs.values() {
        require_digest(value)?;
    }
    Ok(())
}

fn validate_compatibility(c: &Compatibility) -> Result<(), String> {
    if c.system != "x86_64-linux" || c.host_triple != "x86_64-unknown-linux-gnu" || c.target_triple != c.host_triple {
        return Err("cargo-bundle-target-unsupported".to_string());
    }
    if c.store_prefix != "/nix/store" && c.store_prefix != "/mantle/store" {
        return Err("cargo-bundle-prefix-unsupported".to_string());
    }
    if c.sandbox != "native" || c.network != "denied" {
        return Err("cargo-bundle-policy-unsupported".to_string());
    }
    if c.plan_schema != "mantle-plan-v2"
        || c.source_schema != "mantle-source-bundle-v1"
        || c.receipt_schema != "mantle-build-receipt-bundle-v1"
    {
        return Err("cargo-bundle-mandatory-schema-unsupported".to_string());
    }
    if c.max_plan_bytes != 4_194_304 || c.max_units != 4096 || c.max_sources != 256 || c.max_direct_inputs != 256 {
        return Err("cargo-bundle-admission-limits-unsupported".to_string());
    }
    require_digest(&c.policy_blake3)?;
    require_digest(&c.trust_policy_blake3)
}

fn validate_collections(m: &Manifest) -> Result<(), String> {
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
            return Err("cargo-bundle-entry-limit".to_string());
        }
    }
    if m.profile.is_empty() || m.profile.len() > MAX_TEXT_BYTES || m.units.is_empty() || m.package_sources.is_empty() {
        return Err("cargo-bundle-required-input-missing".to_string());
    }
    // This bounds every string, including nested declarations, before downstream effects.
    let value = serde_json::to_value(m).map_err(|error| error.to_string())?;
    validate_value(&value, 0)
}

fn validate_value(value: &serde_json::Value, depth: u32) -> Result<(), String> {
    if depth > 16 {
        return Err("cargo-bundle-nesting-limit".to_string());
    }
    match value {
        serde_json::Value::String(text) if text.len() > MAX_TEXT_BYTES || text.contains('\0') => {
            Err("cargo-bundle-text-limit".to_string())
        }
        serde_json::Value::Array(items) => {
            if items.len() > MAX_ENTRIES {
                return Err("cargo-bundle-entry-limit".to_string());
            }
            for item in items {
                validate_value(item, depth + 1)?;
            }
            Ok(())
        }
        serde_json::Value::Object(items) => {
            if items.len() > MAX_ENTRIES {
                return Err("cargo-bundle-entry-limit".to_string());
            }
            for (key, item) in items {
                if key.len() > MAX_TEXT_BYTES {
                    return Err("cargo-bundle-text-limit".to_string());
                }
                validate_value(item, depth + 1)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_tools(m: &Manifest) -> Result<(), String> {
    for role in ["rustc", "cargo", "toolchain", "producer", "planner", "helper", "linker"] {
        let path = m.tools.get(role).ok_or_else(|| format!("cargo-bundle-tool-missing:{role}"))?;
        require_covered(path, m)?;
    }
    for path in &m.required_store_paths {
        store_root(path, &m.compatibility.store_prefix)?;
    }
    for name in ["out", "plan", "sources"] {
        let path =
            m.producer_outputs.get(name).ok_or_else(|| format!("cargo-bundle-producer-output-missing:{name}"))?;
        require_covered(path, m)?;
    }
    if let Some(vendor) = &m.vendor_store_path {
        require_covered(vendor, m)?;
    }
    Ok(())
}

fn validate_graph(m: &Manifest) -> Result<(), String> {
    if m.roots.is_empty() {
        return Err("cargo-bundle-roots-missing".to_string());
    }
    for root in &m.roots {
        if !m.units.contains_key(root) {
            return Err("cargo-bundle-root-missing".to_string());
        }
    }
    let mut edges = m.units.keys().map(|id| (id.as_str(), 0usize)).collect::<BTreeMap<_, _>>();
    for unit in m.units.values() {
        if let Some(package) = &unit.package_id {
            if !m.package_sources.contains_key(package) {
                return Err("cargo-bundle-package-source-missing".to_string());
            }
        } else if !unit.features.is_empty() || !unit.dependencies.is_empty() {
            return Err("cargo-bundle-auxiliary-unit-invalid".to_string());
        }
        if unit.dependencies.len() > 256 {
            return Err("cargo-bundle-dependency-limit".to_string());
        }
        for dep in &unit.dependencies {
            let count = edges.get_mut(dep.as_str()).ok_or_else(|| "cargo-bundle-dependency-missing".to_string())?;
            *count = count.saturating_add(1);
        }
    }
    let mut dependents =
        edges.into_iter().map(|(id, count)| (id, Vec::with_capacity(count))).collect::<BTreeMap<_, _>>();
    let mut indegree =
        m.units.iter().map(|(id, unit)| (id.as_str(), unit.dependencies.len())).collect::<BTreeMap<_, _>>();
    for (id, unit) in &m.units {
        for dep in &unit.dependencies {
            dependents.get_mut(dep.as_str()).expect("dependency checked").push(id.as_str());
        }
    }
    let mut ready = Vec::with_capacity(m.units.len());
    ready.extend(indegree.iter().filter_map(|(id, count)| (*count == 0).then_some(*id)));
    let mut admitted = 0usize;
    while let Some(id) = ready.pop() {
        admitted = admitted.saturating_add(1);
        for consumer in &dependents[id] {
            let count = indegree.get_mut(consumer).expect("consumer belongs to graph");
            *count = count.checked_sub(1).expect("one reduction per checked edge");
            if *count == 0 {
                ready.push(*consumer);
            }
        }
    }
    if admitted != m.units.len() {
        return Err("cargo-bundle-unit-cycle".to_string());
    }
    Ok(())
}

fn validate_native(m: &Manifest) -> Result<(), String> {
    for (package, native) in &m.native_inputs {
        if native.schema_version != 1 {
            return Err("cargo-bundle-native-schema-unsupported".to_string());
        }
        if !m.package_sources.contains_key(package) {
            return Err("cargo-bundle-native-package-missing".to_string());
        }
        let resolved = resolve_native_inputs(&native.declaration).map_err(|error| error.to_string())?;
        if resolved != native.resolved {
            return Err("cargo-bundle-native-resolution-mismatch".to_string());
        }
        for path in &resolved.inputs {
            require_covered(path, m)?;
        }
    }
    Ok(())
}

fn require_covered(path: &str, m: &Manifest) -> Result<(), String> {
    let root = store_root(path, &m.compatibility.store_prefix)?;
    if !m.required_store_paths.iter().any(|candidate| candidate == root) {
        return Err(format!("cargo-bundle-payload-undeclared:{path}"));
    }
    Ok(())
}

pub fn store_root<'a>(path: &'a str, prefix: &str) -> Result<&'a str, String> {
    let tail = path
        .strip_prefix(prefix)
        .and_then(|tail| tail.strip_prefix('/'))
        .ok_or_else(|| "cargo-bundle-store-path-invalid".to_string())?;
    let root = tail.split('/').next().unwrap_or("");
    let (hash, name) = root.split_once('-').ok_or_else(|| "cargo-bundle-store-path-invalid".to_string())?;
    if hash.len() != 32
        || !hash.bytes().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&byte))
        || name.is_empty()
        || path.split('/').any(|part| part == "." || part == "..")
    {
        return Err("cargo-bundle-store-path-invalid".to_string());
    }
    Ok(&path[..prefix.len() + 1 + root.len()])
}
fn validate_artifact(a: &Artifact) -> Result<(), String> {
    if a.file.is_empty()
        || a.file.starts_with('/')
        || a.file.contains('\\')
        || a.file.split('/').any(|p| p.is_empty() || p == "." || p == "..")
        || a.bytes == 0
    {
        return Err("cargo-bundle-artifact-path-invalid".to_string());
    }
    require_digest(&a.blake3)
}
fn require_digest(value: &str) -> Result<(), String> {
    if !is_hex(value, 64) {
        return Err("cargo-bundle-digest-invalid".to_string());
    }
    Ok(())
}
fn is_hex(value: &str, bytes: usize) -> bool {
    value.len() == bytes && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
