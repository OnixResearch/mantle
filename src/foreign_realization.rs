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
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;

use crate::foreign_derivation_import::ImportReceipt;
use crate::foreign_executable_plan::CACHE_ONLY_PRESERVE_ROUTE;
use crate::foreign_executable_plan::ExecutableNativeUnit;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_executable_plan::import_receipt_digest;
use crate::foreign_executable_plan::validate_foreign_executable_plan;
use crate::foreign_executable_plan::validate_unit_aterm_projection;
use crate::foreign_graph_compiler::CompiledSourceRequirement;
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
