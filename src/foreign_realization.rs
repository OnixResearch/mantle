//! Pure admission for receipt-bound foreign realization.
//!
//! This module performs no file, store, process, clock, or network I/O. The
//! imperative shell may mutate the store only after this admission succeeds.

use std::collections::BTreeMap;
use std::fmt;

use crunch_build::ExecutionProfile;
use crunch_build::execution_profile_digest;
use crunch_build::validate_execution_profile;
use crunch_build::verify_execution_profile_binding;
use data_encoding::HEXLOWER;
use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;

use crate::foreign_derivation_import::ImportReceipt;
use crate::foreign_executable_plan::ExecutableNativeUnit;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_executable_plan::import_receipt_digest;
use crate::foreign_executable_plan::validate_foreign_executable_plan;
use crate::foreign_executable_plan::validate_unit_aterm_projection;

const MAX_REALIZATION_PROFILES: usize = 1_024;
const HDM_BYTES: usize = 32;

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
}
