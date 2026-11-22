//! Pure admission for receipt-bound foreign realization.
// r[impl foreign_derivation_import.realization_adapter]
// r[impl foreign_derivation_import.source_materialization]
//!
//! This module performs no file, store, process, clock, or network I/O. The
//! imperative shell may mutate the store only after this admission succeeds.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

use crunch_build::ExecutionProfile;
use crunch_build::execution_profile_digest;
use crunch_build::validate_execution_profile;
use crunch_build::verify_execution_profile_binding;
use data_encoding::HEXLOWER;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use snix_store::path_info::PathInfo;
use serde::Deserialize;
use serde::Serialize;

use crate::RunError;
use crate::foreign_derivation_import::ImportReceipt;
use crate::foreign_executable_plan::CACHE_INPUTS_BUILD_OR_REUSE_ROUTE;
use crate::foreign_executable_plan::CACHE_INPUTS_BUILD_ROOTS_ROUTE;
use crate::foreign_executable_plan::CACHE_ONLY_PRESERVE_ROUTE;
use crate::foreign_executable_plan::ExecutableNativeUnit;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_executable_plan::import_receipt_digest;
use crate::foreign_executable_plan::validate_foreign_executable_plan;
use crate::foreign_executable_plan::validate_unit_aterm_projection;
use crate::foreign_graph_compiler::CompiledSourceRequirement;
use crate::foreign_realization_receipt::BUILDER_SIGNER_ROLE;
use crate::foreign_realization_receipt::FOREIGN_REALIZATION_RECEIPT_V2_SCHEMA;
use crate::foreign_realization_receipt::ForeignRealizationReceiptV2;
use crate::foreign_realization_receipt::ROOT_DISPOSITION_BUILT;
use crate::foreign_realization_receipt::foreign_realization_receipt_v2_digest;
use crate::source_bundle::FOREIGN_SOURCE_DESCRIPTOR_DIGEST_METADATA as SOURCE_DESCRIPTOR_DIGEST_METADATA;
use crate::source_bundle::FOREIGN_SOURCE_FOREIGN_PATH_METADATA as SOURCE_FOREIGN_PATH_METADATA;
use crate::source_bundle::FOREIGN_SOURCE_PAYLOAD_ID_METADATA as SOURCE_PAYLOAD_ID_METADATA;
use crate::source_bundle::FOREIGN_SOURCE_TARGET_PATH_METADATA as SOURCE_TARGET_PATH_METADATA;
use crate::source_bundle::SourceBundleManifest;
use crate::source_bundle::validate_empty_source_bundle;
use crate::source_bundle::validate_manifest;

const MAX_REALIZATION_PROFILES: usize = 1_024;
const HDM_BYTES: usize = 32;
pub(crate) const FOREIGN_CACHE_CLOSURE_POLICY_SCHEMA: &str = "mantle-foreign-cache-closure-policy-v1";
const FOREIGN_SOURCE_METADATA_KEYS: [&str; 4] = [
    SOURCE_PAYLOAD_ID_METADATA,
    SOURCE_DESCRIPTOR_DIGEST_METADATA,
    SOURCE_FOREIGN_PATH_METADATA,
    SOURCE_TARGET_PATH_METADATA,
];

#[derive(Debug)]
pub(crate) struct ForeignRealizationAdmission<'a> {
    pub(crate) plan: &'a ForeignExecutablePlan,
    pub(crate) import_receipt: &'a ImportReceipt,
    pub(crate) selected_root_node_ids: &'a [String],
    pub(crate) execution_profiles: &'a BTreeMap<String, ExecutionProfile>,
    pub(crate) remote_execution_requested: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct AdmittedForeignUnit {
    pub(crate) node_id: String,
    pub(crate) drv_path: StorePath<String>,
    pub(crate) hdm: [u8; HDM_BYTES],
    pub(crate) derivation: Derivation,
    pub(crate) execution_profile: ExecutionProfile,
}

#[derive(Clone, Debug)]
pub(crate) struct AdmittedForeignRealization {
    pub(crate) selected_root_node_ids: Vec<String>,
    pub(crate) selected_root_paths: Vec<StorePath<String>>,
    pub(crate) units: Vec<AdmittedForeignUnit>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignCacheClosurePolicy {
    pub(crate) schema: String,
    pub(crate) max_paths: usize,
    pub(crate) max_references: usize,
    pub(crate) max_nar_bytes: u64,
    pub(crate) max_depth: usize,
}

#[derive(Debug)]
pub(crate) struct ForeignSourceAdmission<'a> {
    pub(crate) source_requirements: &'a [CompiledSourceRequirement],
    pub(crate) source_bundle: &'a SourceBundleManifest,
    pub(crate) expected_manifest_blake3: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmittedForeignSource {
    pub(crate) payload_id: String,
    pub(crate) foreign_path: String,
    pub(crate) target_path: String,
    pub(crate) descriptor_blake3: String,
    pub(crate) record_index: usize,
    pub(crate) record_identity: String,
    pub(crate) content_blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ForeignRealizationAdmissionError {
    InvalidPlan(String),
    ImportReceiptDigestMismatch,
    ImportReceiptFieldMismatch,
    InvalidSelectedRoots,
    UnknownSelectedRoot(String),
    ProfileCountLimit,
    ProfileKeyMismatch(String),
    InvalidExecutionProfile { profile_id: String, reason: String },
    MissingExecutionProfile(String),
    ExecutionProfileDigestMismatch(String),
    ExecutionProfileBinding { node_id: String, reason: String },
    InvalidNativeUnit { node_id: String, reason: String },
    InvalidHdm(String),
    InvalidTargetPath(String),
    InvalidSourceBundle(String),
    SourceBundleDigestMismatch,
    InvalidForeignSourceMetadata(String),
    DuplicateForeignSource(String),
    UnexpectedForeignSource(String),
    MissingForeignSource(String),
    ForeignSourceBindingMismatch(String),
    InvalidCacheClosurePolicy(String),
    CacheOnlySourceBundleNotEmpty,
    CacheInputsFixedOutputRoot(String),
    CacheInputsRootOffered(String),
    CacheInputsRootAlreadyPresent(String),
    CacheInputsRootInSeedSet(String),
    CacheInputsPathOutsideMap(String),
    RemoteExecutionUnsupported,
}

impl fmt::Display for ForeignRealizationAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlan(reason) => write!(formatter, "foreign executable plan is invalid: {reason}"),
            Self::ImportReceiptDigestMismatch => {
                write!(formatter, "foreign import receipt digest does not match the executable plan")
            }
            Self::ImportReceiptFieldMismatch => {
                write!(formatter, "foreign import receipt fields do not match the executable plan")
            }
            Self::InvalidSelectedRoots => {
                write!(formatter, "foreign realization selected-root set must be sorted, unique, and non-empty")
            }
            Self::UnknownSelectedRoot(node_id) => {
                write!(formatter, "foreign realization selected root is absent from the executable plan: {node_id}")
            }
            Self::ProfileCountLimit => write!(formatter, "foreign realization execution profile set exceeds its bound"),
            Self::ProfileKeyMismatch(profile_id) => {
                write!(formatter, "foreign realization execution profile map key differs from profile ID: {profile_id}")
            }
            Self::InvalidExecutionProfile { profile_id, reason } => {
                write!(formatter, "foreign realization execution profile is invalid: {profile_id}: {reason}")
            }
            Self::MissingExecutionProfile(profile_id) => {
                write!(formatter, "foreign realization native unit has no supplied execution profile: {profile_id}")
            }
            Self::ExecutionProfileDigestMismatch(node_id) => {
                write!(formatter, "foreign realization native unit profile digest is stale: {node_id}")
            }
            Self::ExecutionProfileBinding { node_id, reason } => {
                write!(formatter, "foreign realization native unit profile binding is invalid: {node_id}: {reason}")
            }
            Self::InvalidNativeUnit { node_id, reason } => {
                write!(formatter, "foreign realization native unit derivation is invalid: {node_id}: {reason}")
            }
            Self::InvalidHdm(node_id) => write!(formatter, "foreign realization native unit HDM is invalid: {node_id}"),
            Self::InvalidTargetPath(path) => {
                write!(formatter, "foreign realization native unit target path is invalid: {path}")
            }
            Self::InvalidSourceBundle(reason) => write!(formatter, "foreign source bundle is invalid: {reason}"),
            Self::SourceBundleDigestMismatch => {
                write!(formatter, "foreign source bundle digest does not match the admitted digest")
            }
            Self::InvalidForeignSourceMetadata(identity) => {
                write!(formatter, "foreign source record metadata is incomplete: {identity}")
            }
            Self::DuplicateForeignSource(payload_id) => {
                write!(formatter, "foreign source payload occurs more than once: {payload_id}")
            }
            Self::UnexpectedForeignSource(payload_id) => {
                write!(formatter, "foreign source payload is absent from the executable plan: {payload_id}")
            }
            Self::MissingForeignSource(payload_id) => {
                write!(formatter, "foreign source payload is absent from the admitted source bundle: {payload_id}")
            }
            Self::ForeignSourceBindingMismatch(payload_id) => {
                write!(formatter, "foreign source payload binding does not match the executable plan: {payload_id}")
            }
            Self::InvalidCacheClosurePolicy(reason) => {
                write!(formatter, "foreign cache closure policy is invalid: {reason}")
            }
            Self::CacheOnlySourceBundleNotEmpty => {
                write!(formatter, "cache-only foreign realization requires an empty source bundle")
            }
            Self::CacheInputsFixedOutputRoot(node_id) => {
                write!(formatter, "cache-inputs build route refuses a fixed-output selected root: {node_id}")
            }
            Self::CacheInputsRootOffered(path) => {
                write!(formatter, "cache-inputs build route refuses a root offered by the cache: {path}")
            }
            Self::CacheInputsRootAlreadyPresent(path) => {
                write!(formatter, "cache-inputs build route refuses a root already present in the store: {path}")
            }
            Self::CacheInputsRootInSeedSet(path) => {
                write!(formatter, "cache-inputs build route seed set contains a selected root output: {path}")
            }
            Self::CacheInputsPathOutsideMap(path) => {
                write!(formatter, "cache-inputs build route seed path is outside the exact path map: {path}")
            }
            Self::RemoteExecutionUnsupported => write!(formatter, "remote foreign realization is unsupported"),
        }
    }
}

impl std::error::Error for ForeignRealizationAdmissionError {}

pub(crate) fn validate_foreign_realization_admission(
    admission: ForeignRealizationAdmission<'_>,
) -> Result<AdmittedForeignRealization, ForeignRealizationAdmissionError> {
    if admission.remote_execution_requested {
        return Err(ForeignRealizationAdmissionError::RemoteExecutionUnsupported);
    }
    validate_foreign_executable_plan(admission.plan)
        .map_err(|diagnostic| ForeignRealizationAdmissionError::InvalidPlan(diagnostic.class))?;
    validate_import_receipt_link(admission.plan, admission.import_receipt)?;
    let selected_root_paths = validate_selected_roots(admission.plan, admission.selected_root_node_ids)?;
    if is_cache_inputs_build_route_plan(admission.plan) {
        reject_fixed_output_roots(admission.plan, admission.selected_root_node_ids)?;
    }
    validate_profile_set(admission.execution_profiles)?;
    let units = admission
        .plan
        .native_units
        .iter()
        .map(|unit| admit_unit(admission.plan, unit, admission.execution_profiles))
        .collect::<Result<Vec<_>, _>>()?;
    debug_assert_eq!(units.len(), admission.plan.native_units.len());
    Ok(AdmittedForeignRealization {
        selected_root_node_ids: admission.selected_root_node_ids.to_vec(),
        selected_root_paths,
        units,
    })
}

pub(crate) fn is_cache_only_foreign_plan(plan: &ForeignExecutablePlan) -> bool {
    plan.realization_route.as_deref() == Some(CACHE_ONLY_PRESERVE_ROUTE)
}

pub(crate) fn is_cache_inputs_build_roots_plan(plan: &ForeignExecutablePlan) -> bool {
    plan.realization_route.as_deref() == Some(CACHE_INPUTS_BUILD_ROOTS_ROUTE)
}

pub(crate) fn is_cache_inputs_build_or_reuse_plan(plan: &ForeignExecutablePlan) -> bool {
    plan.realization_route.as_deref() == Some(CACHE_INPUTS_BUILD_OR_REUSE_ROUTE)
}

/// Both cache-inputs build routes share the cache-hydrated admission shape.
pub(crate) fn is_cache_inputs_build_route_plan(plan: &ForeignExecutablePlan) -> bool {
    is_cache_inputs_build_roots_plan(plan) || is_cache_inputs_build_or_reuse_plan(plan)
}

pub(crate) const BUILDER_PUBLIC_KEY_DIGEST_DOMAIN: &[u8] = b"mantle-builder-public-key-v1\0";
const REALIZATION_TRUST_POLICY_DIGEST_DOMAIN: &[u8] = b"mantle-realization-trust-policy-v1\0";

/// One key allowed to attest a Mantle-built root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BuilderKey {
    pub(crate) name: String,
    pub(crate) public_key_blake3: String,
    pub(crate) verifying_key: VerifyingKey,
}

impl BuilderKey {
    pub(crate) fn new(verifying_key: VerifyingKey) -> Self {
        let public_key_blake3 = realization_public_key_blake3(&verifying_key);
        Self {
            name: verifying_key.name().to_string(),
            public_key_blake3,
            verifying_key,
        }
    }
}

/// Lowercase hex BLAKE3 of the key domain plus the 32-byte ed25519 public key.
pub(crate) fn realization_public_key_blake3(verifying_key: &VerifyingKey) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(BUILDER_PUBLIC_KEY_DIGEST_DOMAIN);
    hasher.update(&verifying_key.public_key_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Digest of the builder and cache key material this realization trusts.
pub(crate) fn realization_trust_policy_blake3(
    builder_keys: &[BuilderKey],
    cache_keys: &[VerifyingKey],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct TrustPolicyMaterial {
        builder_keys: Vec<String>,
        cache_keys: Vec<String>,
    }
    let mut builder = builder_keys.iter().map(|key| key.public_key_blake3.clone()).collect::<Vec<_>>();
    builder.sort();
    builder.dedup();
    let mut cache = cache_keys.iter().map(realization_public_key_blake3).collect::<Vec<_>>();
    cache.sort();
    cache.dedup();
    let canonical = serde_json::to_vec(&TrustPolicyMaterial {
        builder_keys: builder,
        cache_keys: cache,
    })
    .map_err(|error| RunError::Internal(format!("serializing realization trust policy: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(REALIZATION_TRUST_POLICY_DIGEST_DOMAIN);
    hasher.update(&canonical);
    Ok(hasher.finalize().to_hex().to_string())
}

/// Why a present root output cannot be reused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PresentRootRefusal {
    NoBuiltReceipt,
    ReceiptInvalid,
    RouteMismatch,
    NotBuilt,
    DerivationMismatch,
    NarHashMismatch,
    NoValidBuilderSignature,
    SignerMismatch,
    BuilderKeyNotTrusted,
}

impl PresentRootRefusal {
    pub(crate) fn reason(self) -> &'static str {
        match self {
            Self::NoBuiltReceipt => "no-built-receipt",
            Self::ReceiptInvalid => "receipt-invalid",
            Self::RouteMismatch => "route-mismatch",
            Self::NotBuilt => "not-built",
            Self::DerivationMismatch => "derivation-mismatch",
            Self::NarHashMismatch => "nar-hash-mismatch",
            Self::NoValidBuilderSignature => "no-valid-builder-signature",
            Self::SignerMismatch => "signer-mismatch",
            Self::BuilderKeyNotTrusted => "builder-key-not-trusted",
        }
    }
}

impl fmt::Display for PresentRootRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "present root cannot be reused: {}", self.reason())
    }
}

/// The indexed original receipt state for one present root.
pub(crate) enum IndexedBuiltReceipt<'a> {
    Absent,
    Unparsable,
    Present(&'a ForeignRealizationReceiptV2),
}

/// Reuse evidence for one present root output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReusedRootFacts {
    pub(crate) original_receipt_blake3: String,
    pub(crate) signer_key_name: String,
    pub(crate) signer_public_key_blake3: String,
    pub(crate) nar_sha256: String,
    pub(crate) nar_size: u64,
    pub(crate) references: Vec<String>,
}

pub(crate) struct PresentRootInput<'a> {
    pub(crate) target_derivation: &'a str,
    pub(crate) output_name: &'a str,
    pub(crate) path: &'a str,
    pub(crate) store_dir: &'a str,
    pub(crate) path_info: &'a PathInfo,
    pub(crate) recomputed_nar_sha256: [u8; 32],
    pub(crate) builder_keys: &'a [BuilderKey],
    pub(crate) indexed: IndexedBuiltReceipt<'a>,
}

/// Decide whether a present root is an earlier Mantle build.
pub(crate) fn classify_present_root(
    input: PresentRootInput<'_>,
) -> Result<ReusedRootFacts, PresentRootRefusal> {
    assert!(!input.store_dir.is_empty(), "store directory must not be empty");
    assert!(!input.path.is_empty(), "root path must not be empty");
    let receipt = match input.indexed {
        IndexedBuiltReceipt::Absent => return Err(PresentRootRefusal::NoBuiltReceipt),
        IndexedBuiltReceipt::Unparsable => return Err(PresentRootRefusal::ReceiptInvalid),
        IndexedBuiltReceipt::Present(receipt) => receipt,
    };
    if receipt.schema != FOREIGN_REALIZATION_RECEIPT_V2_SCHEMA {
        return Err(PresentRootRefusal::ReceiptInvalid);
    }
    if foreign_realization_receipt_v2_digest(receipt).map_err(|_| PresentRootRefusal::ReceiptInvalid)?
        != receipt.receipt_blake3
    {
        return Err(PresentRootRefusal::ReceiptInvalid);
    }
    if receipt.cache_policy.realization_route.as_deref() != Some(CACHE_INPUTS_BUILD_OR_REUSE_ROUTE) {
        return Err(PresentRootRefusal::RouteMismatch);
    }
    let entry = receipt.roots.iter().find(|root| {
        root.target_derivation == input.target_derivation
            && root.output_name == input.output_name
            && root.path == input.path
    });
    let Some(entry) = entry else {
        return Err(PresentRootRefusal::DerivationMismatch);
    };
    if entry.disposition != ROOT_DISPOSITION_BUILT
        || entry.built.is_none()
        || entry.reused.is_some()
        || entry.signer.role != BUILDER_SIGNER_ROLE
    {
        return Err(PresentRootRefusal::NotBuilt);
    }
    let recorded_nar_sha256 = HEXLOWER.encode(&input.path_info.nar_sha256);
    let recomputed_nar_sha256 = HEXLOWER.encode(&input.recomputed_nar_sha256);
    if entry.nar_sha256 != recorded_nar_sha256
        || entry.nar_sha256 != recomputed_nar_sha256
        || entry.nar_size != input.path_info.nar_size
    {
        return Err(PresentRootRefusal::NarHashMismatch);
    }
    if input.builder_keys.is_empty() {
        return Err(PresentRootRefusal::BuilderKeyNotTrusted);
    }
    let original_fingerprint = entry.signer.public_key_blake3.as_str();
    let trusted = input.builder_keys.iter().any(|key| key.public_key_blake3 == original_fingerprint);
    let fingerprint = root_fingerprint(input.path_info, input.store_dir);
    let mut valid = input
        .builder_keys
        .iter()
        .filter(|key| input.path_info.signatures.iter().any(|sig| key.verifying_key.verify(&fingerprint, &sig.as_ref())));
    if valid.clone().next().is_none() {
        return Err(PresentRootRefusal::NoValidBuilderSignature);
    }
    if !trusted || !valid.any(|key| key.public_key_blake3 == original_fingerprint) {
        return Err(PresentRootRefusal::SignerMismatch);
    }
    let mut references = input
        .path_info
        .references
        .iter()
        .map(|reference| reference.to_absolute_path_with_prefix(input.store_dir))
        .collect::<Vec<_>>();
    references.sort();
    references.dedup();
    let mut recorded_references = entry.references.clone();
    recorded_references.sort();
    recorded_references.dedup();
    if references != recorded_references {
        return Err(PresentRootRefusal::DerivationMismatch);
    }
    Ok(ReusedRootFacts {
        original_receipt_blake3: receipt.receipt_blake3.clone(),
        signer_key_name: entry.signer.key_name.clone(),
        signer_public_key_blake3: entry.signer.public_key_blake3.clone(),
        nar_sha256: entry.nar_sha256.clone(),
        nar_size: entry.nar_size,
        references,
    })
}

fn root_fingerprint(path_info: &PathInfo, store_dir: &str) -> String {
    let store_path = path_info.store_path.as_ref();
    let references = path_info.references.iter().map(|reference| reference.as_ref()).collect::<Vec<_>>();
    fingerprint_with_store_dir(&store_path, &path_info.nar_sha256, path_info.nar_size, references.iter(), store_dir)
}

/// A reused root must come back from the scheduler as a cache hit, never a build.
pub(crate) fn reused_root_outcome_is_cache_hit(
    outcomes: &[crunch_pipeline::BuildOutcome],
    target_derivation: &str,
    store_dir: &str,
) -> bool {
    assert!(!target_derivation.is_empty(), "target derivation must not be empty");
    assert!(!store_dir.is_empty(), "store directory must not be empty");
    outcomes.iter().any(|outcome| {
        outcome.drv_path.to_absolute_path_with_prefix(store_dir) == target_derivation
            && outcome.cached
            && outcome.substitutions.is_empty()
    })
}

/// Compute the exact set of non-root paths the selected roots need at build time.
///
/// The set is the outputs requested by each selected root's direct input
/// derivation edges, the roots' own input sources, and nothing else. Runtime
/// references are added by the signed cache closure walk, not here.
pub(crate) fn cache_inputs_closure(
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
) -> Result<Vec<String>, ForeignRealizationAdmissionError> {
    let mut units_by_node = BTreeMap::new();
    let mut units_by_target = BTreeMap::new();
    for unit in &plan.native_units {
        units_by_node.insert(unit.node_id.as_str(), unit);
        units_by_target.insert(unit.target_derivation.as_str(), unit);
    }
    let selected_roots = selected_root_node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for root_id in &selected_roots {
        if !units_by_node.contains_key(*root_id) {
            return Err(ForeignRealizationAdmissionError::UnknownSelectedRoot((*root_id).to_string()));
        }
    }
    let mut root_outputs = BTreeSet::new();
    for root in &plan.selected_roots {
        if selected_roots.contains(root.node_id.as_str()) {
            root_outputs.extend(root.target_outputs.values().cloned());
        }
    }
    if root_outputs.is_empty() {
        return Err(ForeignRealizationAdmissionError::InvalidSelectedRoots);
    }
    let permitted_outputs = plan.exact_path_maps.outputs.values().cloned().collect::<BTreeSet<_>>();
    let permitted_sources = plan.exact_path_maps.sources.values().cloned().collect::<BTreeSet<_>>();
    let mut seed_paths = BTreeSet::new();
    for root_id in &selected_roots {
        let unit = units_by_node
            .get(root_id)
            .ok_or_else(|| ForeignRealizationAdmissionError::UnknownSelectedRoot((*root_id).to_string()))?;
        for (input_derivation, output_names) in &unit.input_derivations {
            let input_unit = units_by_target.get(input_derivation.as_str()).ok_or_else(|| {
                ForeignRealizationAdmissionError::InvalidNativeUnit {
                    node_id: unit.node_id.clone(),
                    reason: format!("input derivation is absent from the plan: {input_derivation}"),
                }
            })?;
            for output_name in output_names {
                if !input_unit.outputs.contains_key(output_name.as_str()) {
                    return Err(ForeignRealizationAdmissionError::InvalidNativeUnit {
                        node_id: input_unit.node_id.clone(),
                        reason: format!("requested output is absent from the plan: {output_name}"),
                    });
                }
            }
            // Mantle realizes a derivation only when every output is present,
            // so a direct input must hydrate all of its outputs, not just the
            // output named by this edge.
            if input_unit.outputs.is_empty() {
                return Err(ForeignRealizationAdmissionError::InvalidNativeUnit {
                    node_id: input_unit.node_id.clone(),
                    reason: "direct input derivation has no outputs".to_string(),
                });
            }
            for target in input_unit.outputs.values() {
                seed_paths.insert(target.clone());
            }
        }
        for source in &unit.input_sources {
            seed_paths.insert(source.clone());
        }
    }
    for path in &seed_paths {
        if root_outputs.contains(path) {
            return Err(ForeignRealizationAdmissionError::CacheInputsRootInSeedSet(path.clone()));
        }
        let is_permitted = permitted_outputs.contains(path) || permitted_sources.contains(path);
        if !is_permitted {
            return Err(ForeignRealizationAdmissionError::CacheInputsPathOutsideMap(path.clone()));
        }
    }
    Ok(seed_paths.into_iter().collect())
}

/// Non-root derivations whose every output is in the admitted hydrated set.
///
/// These are satisfied before scheduling: their goals are never built and their
/// build-only inputs are never expanded into goals.
pub(crate) fn satisfied_input_derivations(
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
    hydrated_paths: &BTreeSet<String>,
) -> BTreeSet<String> {
    let selected_roots = selected_root_node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut satisfied = BTreeSet::new();
    for unit in &plan.native_units {
        if selected_roots.contains(unit.node_id.as_str()) {
            continue;
        }
        if unit.outputs.is_empty() {
            continue;
        }
        if unit.outputs.values().all(|path| hydrated_paths.contains(path)) {
            satisfied.insert(unit.target_derivation.clone());
        }
    }
    satisfied
}

pub(crate) fn reject_fixed_output_roots(
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
) -> Result<(), ForeignRealizationAdmissionError> {
    for unit in &plan.native_units {
        if !selected_root_node_ids.iter().any(|root| root == &unit.node_id) {
            continue;
        }
        if !matches!(&unit.builtin, crate::foreign_graph_compiler::CompiledForeignBuiltin::NativeDerivation) {
            return Err(ForeignRealizationAdmissionError::CacheInputsFixedOutputRoot(unit.node_id.clone()));
        }
    }
    Ok(())
}

pub(crate) fn validate_cache_inputs_roots(
    roots: &[String],
    cache_offered_roots: &[String],
    present_roots: &[String],
) -> Result<(), ForeignRealizationAdmissionError> {
    if let Some(path) = cache_offered_roots.first() {
        return Err(ForeignRealizationAdmissionError::CacheInputsRootOffered(path.clone()));
    }
    if let Some(path) = present_roots.first() {
        return Err(ForeignRealizationAdmissionError::CacheInputsRootAlreadyPresent(path.clone()));
    }
    if roots.is_empty() {
        return Err(ForeignRealizationAdmissionError::InvalidSelectedRoots);
    }
    Ok(())
}

pub(crate) fn validate_foreign_cache_closure_policy(
    policy: &ForeignCacheClosurePolicy,
) -> Result<(), ForeignRealizationAdmissionError> {
    if policy.schema != FOREIGN_CACHE_CLOSURE_POLICY_SCHEMA {
        return Err(ForeignRealizationAdmissionError::InvalidCacheClosurePolicy("unsupported schema".to_string()));
    }
    if policy.max_paths == 0 || policy.max_references == 0 || policy.max_nar_bytes == 0 || policy.max_depth == 0 {
        return Err(ForeignRealizationAdmissionError::InvalidCacheClosurePolicy(
            "all limits must be positive".to_string(),
        ));
    }
    if policy.max_references < policy.max_paths {
        return Err(ForeignRealizationAdmissionError::InvalidCacheClosurePolicy(
            "max_references must be at least max_paths".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_cache_only_source_bundle(
    source_bundle: &SourceBundleManifest,
    expected_manifest_blake3: &str,
) -> Result<(), ForeignRealizationAdmissionError> {
    if !source_bundle.records.is_empty() || !source_bundle.roots.is_empty() {
        return Err(ForeignRealizationAdmissionError::CacheOnlySourceBundleNotEmpty);
    }
    validate_empty_source_bundle(source_bundle)
        .map_err(|error| ForeignRealizationAdmissionError::InvalidSourceBundle(error.to_string()))?;
    if source_bundle.manifest_blake3 != expected_manifest_blake3 {
        return Err(ForeignRealizationAdmissionError::SourceBundleDigestMismatch);
    }
    Ok(())
}

pub(crate) fn validate_foreign_source_admission(
    admission: ForeignSourceAdmission<'_>,
) -> Result<Vec<AdmittedForeignSource>, ForeignRealizationAdmissionError> {
    validate_manifest(admission.source_bundle)
        .map_err(|error| ForeignRealizationAdmissionError::InvalidSourceBundle(error.to_string()))?;
    if admission.source_bundle.manifest_blake3 != admission.expected_manifest_blake3 {
        return Err(ForeignRealizationAdmissionError::SourceBundleDigestMismatch);
    }
    let requirements = admission
        .source_requirements
        .iter()
        .map(|requirement| (requirement.payload_id.as_str(), requirement))
        .collect::<BTreeMap<_, _>>();
    let expected_payload_ids = requirements.keys().copied().collect::<BTreeSet<_>>();
    if expected_payload_ids.len() != admission.source_requirements.len() {
        return Err(ForeignRealizationAdmissionError::InvalidPlan(
            "foreign executable plan repeats a source payload ID".to_string(),
        ));
    }
    let foreign_paths = admission
        .source_requirements
        .iter()
        .map(|requirement| requirement.foreign_path.as_str())
        .collect::<BTreeSet<_>>();
    if foreign_paths.len() != admission.source_requirements.len() {
        return Err(ForeignRealizationAdmissionError::InvalidPlan(
            "foreign executable plan repeats a foreign source path".to_string(),
        ));
    }
    let target_paths = admission
        .source_requirements
        .iter()
        .map(|requirement| requirement.target_path.as_str())
        .collect::<BTreeSet<_>>();
    if target_paths.len() != admission.source_requirements.len() {
        return Err(ForeignRealizationAdmissionError::InvalidPlan(
            "foreign executable plan repeats a target source path".to_string(),
        ));
    }

    let mut admitted_by_payload = BTreeMap::<String, AdmittedForeignSource>::new();
    for (record_index, record) in admission.source_bundle.records.iter().enumerate() {
        let present_metadata_count =
            FOREIGN_SOURCE_METADATA_KEYS.iter().filter(|key| record.metadata.contains_key(**key)).count();
        if present_metadata_count == 0 {
            continue;
        }
        if present_metadata_count != FOREIGN_SOURCE_METADATA_KEYS.len() {
            return Err(ForeignRealizationAdmissionError::InvalidForeignSourceMetadata(record.identity.clone()));
        }
        let payload_id = &record.metadata[SOURCE_PAYLOAD_ID_METADATA];
        let requirement = requirements
            .get(payload_id.as_str())
            .copied()
            .ok_or_else(|| ForeignRealizationAdmissionError::UnexpectedForeignSource(payload_id.clone()))?;
        let binding_matches = record.metadata[SOURCE_DESCRIPTOR_DIGEST_METADATA] == requirement.descriptor_digest
            && record.metadata[SOURCE_FOREIGN_PATH_METADATA] == requirement.foreign_path
            && record.metadata[SOURCE_TARGET_PATH_METADATA] == requirement.target_path;
        if !binding_matches {
            return Err(ForeignRealizationAdmissionError::ForeignSourceBindingMismatch(payload_id.clone()));
        }
        let admitted = AdmittedForeignSource {
            payload_id: payload_id.clone(),
            foreign_path: requirement.foreign_path.clone(),
            target_path: requirement.target_path.clone(),
            descriptor_blake3: requirement.descriptor_digest.clone(),
            record_index,
            record_identity: record.identity.clone(),
            content_blake3: record.content_blake3.clone(),
        };
        if admitted_by_payload.insert(payload_id.clone(), admitted).is_some() {
            return Err(ForeignRealizationAdmissionError::DuplicateForeignSource(payload_id.clone()));
        }
    }

    admission
        .source_requirements
        .iter()
        .map(|requirement| {
            admitted_by_payload
                .remove(&requirement.payload_id)
                .ok_or_else(|| ForeignRealizationAdmissionError::MissingForeignSource(requirement.payload_id.clone()))
        })
        .collect()
}

fn validate_import_receipt_link(
    plan: &ForeignExecutablePlan,
    receipt: &ImportReceipt,
) -> Result<(), ForeignRealizationAdmissionError> {
    let digest = import_receipt_digest(receipt)
        .map_err(|diagnostic| ForeignRealizationAdmissionError::InvalidPlan(diagnostic.class))?;
    if digest != plan.accepted_import.receipt_digest_blake3 {
        return Err(ForeignRealizationAdmissionError::ImportReceiptDigestMismatch);
    }
    let package_index_digest = receipt.package_index_digest.as_deref();
    let fields_match = receipt.schema == plan.accepted_import.receipt_schema
        && receipt.producer_identity == plan.accepted_import.producer_identity
        && receipt.raw_graph_digest == plan.accepted_import.raw_graph_digest_blake3
        && receipt.translation_policy_digest == plan.accepted_import.translation_policy_digest_blake3
        && receipt.translated_graph_digest == plan.accepted_import.translated_graph_digest_blake3
        && package_index_digest == Some(plan.accepted_import.package_index_digest_blake3.as_str());
    if !fields_match {
        return Err(ForeignRealizationAdmissionError::ImportReceiptFieldMismatch);
    }
    Ok(())
}

fn validate_selected_roots(
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
) -> Result<Vec<StorePath<String>>, ForeignRealizationAdmissionError> {
    if selected_root_node_ids.is_empty() || !selected_root_node_ids.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(ForeignRealizationAdmissionError::InvalidSelectedRoots);
    }
    let roots = plan.selected_roots.iter().map(|root| (root.node_id.as_str(), root)).collect::<BTreeMap<_, _>>();
    selected_root_node_ids
        .iter()
        .map(|node_id| {
            let root = roots
                .get(node_id.as_str())
                .copied()
                .ok_or_else(|| ForeignRealizationAdmissionError::UnknownSelectedRoot(node_id.clone()))?;
            parse_target_path(&root.target_derivation, &plan.target_store_prefix)
        })
        .collect()
}

fn validate_profile_set(profiles: &BTreeMap<String, ExecutionProfile>) -> Result<(), ForeignRealizationAdmissionError> {
    if profiles.is_empty() || profiles.len() > MAX_REALIZATION_PROFILES {
        return Err(ForeignRealizationAdmissionError::ProfileCountLimit);
    }
    for (profile_id, profile) in profiles {
        if profile_id != &profile.profile_id {
            return Err(ForeignRealizationAdmissionError::ProfileKeyMismatch(profile_id.clone()));
        }
        validate_execution_profile(profile).map_err(|error| {
            ForeignRealizationAdmissionError::InvalidExecutionProfile {
                profile_id: profile_id.clone(),
                reason: error.to_string(),
            }
        })?;
    }
    Ok(())
}

fn admit_unit(
    plan: &ForeignExecutablePlan,
    unit: &ExecutableNativeUnit,
    profiles: &BTreeMap<String, ExecutionProfile>,
) -> Result<AdmittedForeignUnit, ForeignRealizationAdmissionError> {
    let profile = profiles
        .get(&unit.execution_profile_id)
        .ok_or_else(|| ForeignRealizationAdmissionError::MissingExecutionProfile(unit.execution_profile_id.clone()))?;
    let observed_digest = execution_profile_digest(profile).map_err(|error| {
        ForeignRealizationAdmissionError::InvalidExecutionProfile {
            profile_id: profile.profile_id.clone(),
            reason: error.to_string(),
        }
    })?;
    if observed_digest != unit.execution_profile_digest_blake3 {
        return Err(ForeignRealizationAdmissionError::ExecutionProfileDigestMismatch(unit.node_id.clone()));
    }
    let derivation = validate_unit_aterm_projection(plan, unit).map_err(|diagnostic| {
        ForeignRealizationAdmissionError::InvalidNativeUnit {
            node_id: unit.node_id.clone(),
            reason: diagnostic.class,
        }
    })?;
    verify_execution_profile_binding(&derivation, profile).map_err(|error| {
        ForeignRealizationAdmissionError::ExecutionProfileBinding {
            node_id: unit.node_id.clone(),
            reason: error.to_string(),
        }
    })?;
    let hdm = decode_hdm(&unit.hdm_blake3, &unit.node_id)?;
    let drv_path = parse_target_path(&unit.target_derivation, &plan.target_store_prefix)?;
    Ok(AdmittedForeignUnit {
        node_id: unit.node_id.clone(),
        drv_path,
        hdm,
        derivation,
        execution_profile: profile.clone(),
    })
}

fn decode_hdm(value: &str, node_id: &str) -> Result<[u8; HDM_BYTES], ForeignRealizationAdmissionError> {
    let bytes = HEXLOWER
        .decode(value.as_bytes())
        .map_err(|_| ForeignRealizationAdmissionError::InvalidHdm(node_id.to_string()))?;
    bytes.try_into().map_err(|_| ForeignRealizationAdmissionError::InvalidHdm(node_id.to_string()))
}

fn parse_target_path(
    value: &str,
    target_store_prefix: &str,
) -> Result<StorePath<String>, ForeignRealizationAdmissionError> {
    StorePath::from_absolute_path_with_prefix(value.as_bytes(), target_store_prefix)
        .map_err(|_| ForeignRealizationAdmissionError::InvalidTargetPath(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foreign_derivation_import::ForeignDerivationGraph;
    use crate::foreign_derivation_import::PackageIndex;
    use crate::foreign_derivation_import::TranslationPolicy;
    use crate::foreign_executable_plan::compile_foreign_executable_plan;
    use crate::source_bundle::SourceRecordKind;
    use crate::source_bundle::SourceSpec;
    use crate::source_bundle::digest_manifest_without_digest;
    use crate::source_bundle::digest_source_record_content;
    use crate::source_bundle::plan_source_bundle;

    const NIX_GRAPH: &str = include_str!("../tests/fixtures/foreign-import/nix-hello.graph.json");
    const NIX_INDEX: &str = include_str!("../tests/fixtures/foreign-import/nix-hello.index.json");
    const POLICY: &str = include_str!("../tests/fixtures/foreign-import/policy.json");

    #[test]
    fn admission_validates_receipt_roots_profiles_and_native_units() {
        let (plan, receipt) = fixture();
        let roots = vec![plan.selected_roots[0].node_id.clone()];
        let profiles = profile_map(&plan);

        let admitted = validate_foreign_realization_admission(ForeignRealizationAdmission {
            plan: &plan,
            import_receipt: &receipt,
            selected_root_node_ids: &roots,
            execution_profiles: &profiles,
            remote_execution_requested: false,
        })
        .unwrap();

        assert_eq!(admitted.selected_root_node_ids, roots);
        assert_eq!(admitted.selected_root_paths.len(), roots.len());
        assert_eq!(admitted.units.len(), plan.native_units.len());
        assert_eq!(admitted.units[0].node_id, plan.native_units[0].node_id);
    }

    #[test]
    fn admission_rejects_stale_receipt_missing_profile_and_unknown_root() {
        let (plan, mut receipt) = fixture();
        let roots = vec![plan.selected_roots[0].node_id.clone()];
        let profiles = profile_map(&plan);
        receipt.raw_graph_digest = "0".repeat(64);
        assert_eq!(
            validate_foreign_realization_admission(ForeignRealizationAdmission {
                plan: &plan,
                import_receipt: &receipt,
                selected_root_node_ids: &roots,
                execution_profiles: &profiles,
                remote_execution_requested: false,
            })
            .unwrap_err(),
            ForeignRealizationAdmissionError::ImportReceiptDigestMismatch
        );

        let (plan, receipt) = fixture();
        assert!(matches!(
            validate_foreign_realization_admission(ForeignRealizationAdmission {
                plan: &plan,
                import_receipt: &receipt,
                selected_root_node_ids: &roots,
                execution_profiles: &BTreeMap::new(),
                remote_execution_requested: false,
            }),
            Err(ForeignRealizationAdmissionError::ProfileCountLimit)
        ));

        let unknown = vec!["unknown-root".to_string()];
        assert!(matches!(
            validate_foreign_realization_admission(ForeignRealizationAdmission {
                plan: &plan,
                import_receipt: &receipt,
                selected_root_node_ids: &unknown,
                execution_profiles: &profiles,
                remote_execution_requested: false,
            }),
            Err(ForeignRealizationAdmissionError::UnknownSelectedRoot(_))
        ));
    }

    #[test]
    fn source_admission_binds_payloads_to_manifest_records() {
        let (plan, _) = fixture();
        let (_temporary, manifest) = source_bundle_fixture(&plan);

        let admitted = validate_foreign_source_admission(ForeignSourceAdmission {
            source_requirements: &plan.source_requirements,
            source_bundle: &manifest,
            expected_manifest_blake3: &manifest.manifest_blake3,
        })
        .unwrap();

        assert_eq!(admitted.len(), plan.source_requirements.len());
        assert_eq!(admitted[0].payload_id, plan.source_requirements[0].payload_id);
        assert_eq!(admitted[0].content_blake3, manifest.records[admitted[0].record_index].content_blake3);
    }

    #[test]
    fn source_admission_rejects_missing_incomplete_and_stale_bindings() {
        let (plan, _) = fixture();
        let (_temporary, manifest) = source_bundle_fixture(&plan);

        let mut missing = manifest.clone();
        missing.records[0].metadata.clear();
        refresh_source_bundle_digests(&mut missing);
        assert!(matches!(
            validate_foreign_source_admission(ForeignSourceAdmission {
                source_requirements: &plan.source_requirements,
                source_bundle: &missing,
                expected_manifest_blake3: &missing.manifest_blake3,
            }),
            Err(ForeignRealizationAdmissionError::MissingForeignSource(_))
        ));

        let mut incomplete = manifest.clone();
        incomplete.records[0].metadata.remove(SOURCE_TARGET_PATH_METADATA);
        refresh_source_bundle_digests(&mut incomplete);
        assert!(matches!(
            validate_foreign_source_admission(ForeignSourceAdmission {
                source_requirements: &plan.source_requirements,
                source_bundle: &incomplete,
                expected_manifest_blake3: &incomplete.manifest_blake3,
            }),
            Err(ForeignRealizationAdmissionError::InvalidForeignSourceMetadata(_))
        ));

        let mut stale = manifest.clone();
        stale.records[0].metadata.insert(SOURCE_DESCRIPTOR_DIGEST_METADATA.to_string(), "0".repeat(64));
        refresh_source_bundle_digests(&mut stale);
        assert!(matches!(
            validate_foreign_source_admission(ForeignSourceAdmission {
                source_requirements: &plan.source_requirements,
                source_bundle: &stale,
                expected_manifest_blake3: &stale.manifest_blake3,
            }),
            Err(ForeignRealizationAdmissionError::ForeignSourceBindingMismatch(_))
        ));

        let mut repeated_target_requirements = plan.source_requirements.clone();
        let mut repeated = repeated_target_requirements[0].clone();
        repeated.payload_id.push_str("-duplicate");
        repeated.foreign_path.push_str("-duplicate");
        repeated_target_requirements.push(repeated);
        assert!(matches!(
            validate_foreign_source_admission(ForeignSourceAdmission {
                source_requirements: &repeated_target_requirements,
                source_bundle: &manifest,
                expected_manifest_blake3: &manifest.manifest_blake3,
            }),
            Err(ForeignRealizationAdmissionError::InvalidPlan(message))
                if message.contains("repeats a target source path")
        ));
    }

    #[test]
    fn cache_only_policy_and_empty_source_bundle_fail_closed() {
        const MAX_PATHS: usize = 8;
        const MAX_REFERENCES: usize = 16;
        const MAX_NAR_BYTES: u64 = 1_024;
        const MAX_DEPTH: usize = 4;
        let policy = ForeignCacheClosurePolicy {
            schema: FOREIGN_CACHE_CLOSURE_POLICY_SCHEMA.to_string(),
            max_paths: MAX_PATHS,
            max_references: MAX_REFERENCES,
            max_nar_bytes: MAX_NAR_BYTES,
            max_depth: MAX_DEPTH,
        };
        validate_foreign_cache_closure_policy(&policy).unwrap();
        let empty = crate::source_bundle::plan_empty_source_bundle("/nix/store").unwrap();
        validate_cache_only_source_bundle(&empty, &empty.manifest_blake3).unwrap();

        let mut zero = policy.clone();
        zero.max_paths = 0;
        assert!(matches!(
            validate_foreign_cache_closure_policy(&zero),
            Err(ForeignRealizationAdmissionError::InvalidCacheClosurePolicy(_))
        ));
        let temporary = tempfile::tempdir().unwrap();
        let payload = temporary.path().join("payload");
        std::fs::write(&payload, b"payload").unwrap();
        let nonempty = plan_source_bundle(
            &[SourceSpec {
                kind: SourceRecordKind::LocalPath,
                identity: "payload".to_string(),
                path: payload,
                adapter: None,
            }],
            "/nix/store",
        )
        .unwrap();
        assert_eq!(
            validate_cache_only_source_bundle(&nonempty, &nonempty.manifest_blake3).unwrap_err(),
            ForeignRealizationAdmissionError::CacheOnlySourceBundleNotEmpty
        );
    }

    #[test]
    fn admission_rejects_remote_execution_before_unit_admission() {
        let (plan, receipt) = fixture();
        let roots = vec![plan.selected_roots[0].node_id.clone()];
        let profiles = profile_map(&plan);
        assert_eq!(
            validate_foreign_realization_admission(ForeignRealizationAdmission {
                plan: &plan,
                import_receipt: &receipt,
                selected_root_node_ids: &roots,
                execution_profiles: &profiles,
                remote_execution_requested: true,
            })
            .unwrap_err(),
            ForeignRealizationAdmissionError::RemoteExecutionUnsupported
        );
    }

    #[test]
    fn cache_inputs_closure_excludes_roots_and_admits_only_mapped_paths() {
        let (plan, _) = fixture();
        let roots = vec![plan.selected_roots[0].node_id.clone()];
        let closure = cache_inputs_closure(&plan, &roots).unwrap();
        assert!(!closure.is_empty());
        let root_outputs = plan.selected_roots[0].target_outputs.values().cloned().collect::<BTreeSet<_>>();
        let permitted = plan
            .exact_path_maps
            .outputs
            .values()
            .chain(plan.exact_path_maps.sources.values())
            .cloned()
            .collect::<BTreeSet<_>>();
        for path in &closure {
            assert!(!root_outputs.contains(path));
            assert!(permitted.contains(path));
        }
        let mut sorted = closure.clone();
        sorted.sort();
        assert_eq!(closure, sorted);

        assert!(matches!(
            cache_inputs_closure(&plan, &["absent-node".to_string()]),
            Err(ForeignRealizationAdmissionError::UnknownSelectedRoot(_))
        ));
    }

    #[test]
    fn cache_inputs_seeds_every_output_of_a_multi_output_direct_input() {
        let graph: ForeignDerivationGraph =
            serde_json::from_str(include_str!("../tests/fixtures/foreign-import/realize-two-node.graph.json"))
                .expect("graph fixture");
        let index: PackageIndex =
            serde_json::from_str(include_str!("../tests/fixtures/foreign-import/realize-two-node.index.json"))
                .expect("index fixture");
        let policy: TranslationPolicy =
            serde_json::from_str(include_str!("../tests/fixtures/foreign-import/realize-policy.json"))
                .expect("policy fixture");
        let (mut plan, _) = compile_foreign_executable_plan(&graph, &index, &policy, "two-node", "x86_64-linux")
            .expect("two-node plan");
        const DEV_PATH: &str = "/mantle/store/99999999999999999999999999999999-child-dev";
        let child_target = plan
            .native_units
            .iter()
            .find(|unit| unit.input_derivations.is_empty())
            .expect("child unit")
            .target_derivation
            .clone();
        for unit in &mut plan.native_units {
            if unit.target_derivation == child_target {
                unit.outputs.insert("dev".to_string(), DEV_PATH.to_string());
            }
        }
        plan.exact_path_maps.outputs.insert("/gnu/store/child-dev".to_string(), DEV_PATH.to_string());
        let roots = vec![plan.selected_roots[0].node_id.clone()];
        let closure = cache_inputs_closure(&plan, &roots).expect("closure");
        assert!(closure.contains(&DEV_PATH.to_string()), "every output of a direct input must be seeded");
        assert!(closure.iter().any(|path| path.ends_with("-child")));
    }

    #[test]
    fn cache_inputs_refuses_an_edge_naming_an_undeclared_output() {
        let graph: ForeignDerivationGraph =
            serde_json::from_str(include_str!("../tests/fixtures/foreign-import/realize-two-node.graph.json"))
                .expect("graph fixture");
        let index: PackageIndex =
            serde_json::from_str(include_str!("../tests/fixtures/foreign-import/realize-two-node.index.json"))
                .expect("index fixture");
        let policy: TranslationPolicy =
            serde_json::from_str(include_str!("../tests/fixtures/foreign-import/realize-policy.json"))
                .expect("policy fixture");
        let (mut plan, _) = compile_foreign_executable_plan(&graph, &index, &policy, "two-node", "x86_64-linux")
            .expect("two-node plan");
        let root_id = plan.selected_roots[0].node_id.clone();
        let root = plan.native_units.iter_mut().find(|unit| unit.node_id == root_id).expect("root unit");
        let edge = root.input_derivations.values_mut().next().expect("root has an input derivation");
        edge.push("undeclared".to_string());

        let error = cache_inputs_closure(&plan, &[root_id]).expect_err("undeclared edge output must be refused");
        assert!(error.to_string().contains("requested output is absent from the plan: undeclared"), "{error}");
    }

    #[test]
    fn cache_inputs_refuses_a_fixed_output_root_and_a_seeded_root_output() {
        let (mut plan, _) = fixture();
        let root_id = plan.selected_roots[0].node_id.clone();
        let root_target = plan.selected_roots[0].target_derivation.clone();
        for unit in &mut plan.native_units {
            if unit.node_id == root_id {
                unit.builtin = crate::foreign_graph_compiler::CompiledForeignBuiltin::FixedOutput;
            }
        }
        assert!(matches!(
            reject_fixed_output_roots(&plan, std::slice::from_ref(&root_id)),
            Err(ForeignRealizationAdmissionError::CacheInputsFixedOutputRoot(_))
        ));

        let (mut plan, _) = fixture();
        let root_unit = plan.native_units.iter().find(|unit| unit.node_id == root_id).expect("root unit").clone();
        let root_outputs = plan.selected_roots[0].target_outputs.values().cloned().collect::<Vec<_>>();
        for unit in &mut plan.native_units {
            if unit.node_id == root_id {
                unit.input_derivations.insert(root_target.clone(), vec!["out".to_string()]);
                unit.outputs.insert("out".to_string(), root_outputs[0].clone());
            }
        }
        assert!(matches!(
            cache_inputs_closure(&plan, std::slice::from_ref(&root_id)),
            Err(ForeignRealizationAdmissionError::CacheInputsRootInSeedSet(_))
        ));
        assert!(!root_unit.outputs.is_empty());
    }

    #[test]
    fn cache_inputs_root_refusals_report_the_first_path() {
        let roots = vec!["/nix/store/root".to_string()];
        assert!(validate_cache_inputs_roots(&roots, &[], &[]).is_ok());
        assert!(matches!(
            validate_cache_inputs_roots(&roots, &["/nix/store/offered".to_string()], &[]),
            Err(ForeignRealizationAdmissionError::CacheInputsRootOffered(_))
        ));
        assert!(matches!(
            validate_cache_inputs_roots(&roots, &[], &["/nix/store/present".to_string()]),
            Err(ForeignRealizationAdmissionError::CacheInputsRootAlreadyPresent(_))
        ));
        assert!(matches!(
            validate_cache_inputs_roots(&[], &[], &[]),
            Err(ForeignRealizationAdmissionError::InvalidSelectedRoots)
        ));
    }

    fn fixture() -> (ForeignExecutablePlan, ImportReceipt) {
        let graph: ForeignDerivationGraph = serde_json::from_str(NIX_GRAPH).unwrap();
        let index: PackageIndex = serde_json::from_str(NIX_INDEX).unwrap();
        let policy: TranslationPolicy = serde_json::from_str(POLICY).unwrap();
        compile_foreign_executable_plan(&graph, &index, &policy, "hello", "x86_64-linux").unwrap()
    }

    fn profile_map(plan: &ForeignExecutablePlan) -> BTreeMap<String, ExecutionProfile> {
        let profile = crunch_build::foreign_profile_for_producer("nix");
        assert_eq!(execution_profile_digest(&profile).unwrap(), plan.native_units[0].execution_profile_digest_blake3);
        BTreeMap::from([(profile.profile_id.clone(), profile)])
    }

    fn source_bundle_fixture(plan: &ForeignExecutablePlan) -> (tempfile::TempDir, SourceBundleManifest) {
        assert!(!plan.source_requirements.is_empty());
        let temporary = tempfile::tempdir().unwrap();
        let specs = plan
            .source_requirements
            .iter()
            .enumerate()
            .map(|(index, requirement)| {
                let path = temporary.path().join(format!("payload-{index}"));
                std::fs::write(&path, requirement.payload_id.as_bytes()).unwrap();
                SourceSpec {
                    kind: SourceRecordKind::LocalPath,
                    identity: requirement.payload_id.clone(),
                    path,
                    adapter: None,
                }
            })
            .collect::<Vec<_>>();
        let mut manifest = plan_source_bundle(&specs, "/mantle/store").unwrap();
        for record in &mut manifest.records {
            let requirement = plan
                .source_requirements
                .iter()
                .find(|requirement| requirement.payload_id == record.identity)
                .unwrap();
            record.metadata.insert(SOURCE_PAYLOAD_ID_METADATA.to_string(), requirement.payload_id.clone());
            record
                .metadata
                .insert(SOURCE_DESCRIPTOR_DIGEST_METADATA.to_string(), requirement.descriptor_digest.clone());
            record.metadata.insert(SOURCE_FOREIGN_PATH_METADATA.to_string(), requirement.foreign_path.clone());
            record.metadata.insert(SOURCE_TARGET_PATH_METADATA.to_string(), requirement.target_path.clone());
        }
        refresh_source_bundle_digests(&mut manifest);
        (temporary, manifest)
    }

    fn refresh_source_bundle_digests(manifest: &mut SourceBundleManifest) {
        for record in &mut manifest.records {
            record.content_blake3 =
                digest_source_record_content(&record.kind, &record.metadata, &record.files).unwrap();
        }
        manifest.manifest_blake3 = digest_manifest_without_digest(manifest).unwrap();
    }
}
