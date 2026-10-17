// r[impl store_transports.pathinfo_final_nar_migration]
use std::path::Path;
use std::path::PathBuf;

use crunch_attestation::ArtifactAttestation;
use crunch_attestation::Canonicalize;
pub use crunch_repair_core::FinalNarFacts;
pub use crunch_repair_core::FinalNarRepairPlan;
pub use crunch_repair_core::FinalNarRepairPlanningFacts;
pub use crunch_repair_core::FinalNarRepairRejection;
use crunch_repair_core::RepairExecutionMode;
use crunch_repair_core::RepairMutationIntent;
use crunch_repair_core::RepairReportRequest;
use crunch_repair_core::RepairReportStatus;
use crunch_repair_core::RepairRollbackIntent;
use crunch_repair_core::RepairSidecarDisposition;
use crunch_repair_core::RepairTransactionPlan;
use crunch_repair_core::RepairTransactionRequest;
pub use crunch_repair_core::SHA256_DIGEST_BYTES;
pub use crunch_repair_core::plan_final_nar_repair;
use crunch_repair_core::plan_repair_report;
use crunch_repair_core::plan_repair_transaction;
use nix_compat::narinfo::Signature;
use nix_compat::narinfo::SigningKey;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use serde::Serialize;
use snix_store::nar::NarCalculationService;
use snix_store::nar::SimpleRenderer;
use snix_store::path_info::PathInfo;
use tokio::io::AsyncWriteExt;

use crate::Error;
use crate::StoreHandle;
use crate::artifact_attestation_file_path;
use crate::path_identity::require_ca_path_identity;

const SIGNATURE_COUNT_MAX: usize = 4_096;
const ATTESTATION_REPAIR_TEMP_EXTENSION: &str = "json.final-nar-repair.tmp";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinalNarRepairStatus {
    Current,
    WouldRepair,
    Repaired,
}

impl FinalNarRepairStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::WouldRepair => "would-repair",
            Self::Repaired => "repaired",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactAttestationRepairStatus {
    Absent,
    Current,
    WouldRefresh,
    Refreshed,
}

impl ArtifactAttestationRepairStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Current => "current",
            Self::WouldRefresh => "would-refresh",
            Self::Refreshed => "refreshed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FinalNarRepairReport {
    pub format: &'static str,
    pub store_path: String,
    pub status: FinalNarRepairStatus,
    pub execution_requested: bool,
    pub mutated: bool,
    pub recorded_nar_size: u64,
    pub recorded_nar_sha256: String,
    pub observed_nar_size: u64,
    pub observed_nar_sha256: String,
    pub old_signature_count: u32,
    pub new_signature_count: u32,
    pub signer: Option<String>,
    pub artifact_attestation: ArtifactAttestationRepairStatus,
    pub non_claim: &'static str,
}

#[derive(Clone, Debug)]
pub struct FinalNarRepairInspection {
    original_path_info: PathInfo,
    observed: FinalNarFacts,
    transaction_plan: RepairTransactionPlan,
    artifact_attestation: Option<ArtifactAttestation>,
    signature_count: u32,
    store_dir: String,
}

impl FinalNarRepairInspection {
    pub fn is_repair_required(&self) -> bool {
        self.transaction_plan.final_nar_plan == FinalNarRepairPlan::Repair
    }

    pub fn report(&self, is_execution_requested: bool) -> FinalNarRepairReport {
        let execution_mode = if is_execution_requested {
            RepairExecutionMode::Execute
        } else {
            RepairExecutionMode::Inspect
        };
        let decision = plan_repair_report(RepairReportRequest {
            final_nar_plan: self.transaction_plan.final_nar_plan,
            artifact_attestation_present: self.artifact_attestation.is_some(),
            execution_mode,
            execution_completed: false,
        });
        repair_report(RepairReportInput {
            original: &self.original_path_info,
            store_dir: &self.store_dir,
            observed: self.observed,
            status: shell_report_status(decision.status),
            is_execution_requested: decision.execution_requested,
            is_mutated: decision.mutated,
            old_signature_count: self.signature_count,
            signer: None,
            artifact_attestation: shell_sidecar_status(decision.sidecar_disposition, false),
        })
    }
}

fn require_final_nar_repair(handle: &StoreHandle, profile: crate::StoreBackendCapabilityProfile) -> Result<(), Error> {
    if profile.core.contains(&"store-repair-final-nar") {
        Ok(())
    } else {
        Err(Error::Store(format!("{}-repair-final-nar-unsupported", handle.backend().as_str())))
    }
}

pub async fn inspect_final_nar_repair(
    handle: &StoreHandle,
    logical_store_path: &str,
) -> Result<FinalNarRepairInspection, Error> {
    inspect_final_nar_repair_with_profile(handle, logical_store_path, handle.backend().profile()).await
}

async fn inspect_final_nar_repair_with_profile(
    handle: &StoreHandle,
    logical_store_path: &str,
    profile: crate::StoreBackendCapabilityProfile,
) -> Result<FinalNarRepairInspection, Error> {
    require_final_nar_repair(handle, profile)?;
    assert!(handle.store_dir().starts_with('/'), "logical store prefix must be absolute");
    assert!(!handle.state_dir().as_os_str().is_empty(), "state directory must not be empty");
    handle.revalidate_overlay_bases()?;
    let store_path = parse_exact_store_path(ExactStorePathInput {
        logical_store_path,
        store_dir: handle.store_dir(),
    })?;
    let original_path_info = load_exact_pathinfo(handle, &store_path).await?;
    if original_path_info.signatures.len() > SIGNATURE_COUNT_MAX {
        return Err(repair_error(format!("signature count exceeds {SIGNATURE_COUNT_MAX}")));
    }
    let is_content_complete = handle.castore_has_complete_content(&original_path_info.node).await?;
    if !is_content_complete {
        return Err(repair_error("local castore content is missing or incomplete"));
    }
    let is_ca_path_identity_valid = require_ca_path_identity(&original_path_info, handle.store_dir()).is_ok();
    if !is_ca_path_identity_valid {
        return Err(repair_error("CA metadata does not derive the selected store path"));
    }
    let observed = calculate_final_nar_facts(handle, &original_path_info).await?;
    let artifact_attestation = handle.get_artifact_attestation(&original_path_info.store_path).await?;
    let artifact_attestation = artifact_attestation.map(|stored| stored.attestation);
    let is_attestation_valid =
        validate_artifact_attestation(artifact_attestation.as_ref(), &original_path_info, handle.store_dir());
    let signature_count = u32::try_from(original_path_info.signatures.len())
        .map_err(|_| repair_error("signature count does not fit report bounds"))?;
    handle.revalidate_overlay_bases()?;
    let transaction_plan = plan_repair_transaction(RepairTransactionRequest {
        path_id: original_path_info.store_path.to_absolute_path_with_prefix(handle.store_dir()),
        planning_facts: FinalNarRepairPlanningFacts {
            recorded: path_info_final_nar_facts(&original_path_info),
            observed,
            signature_count,
            is_content_complete,
            is_ca_path_identity_valid,
            is_attestation_valid,
        },
        artifact_attestation_present: artifact_attestation.is_some(),
    })
    .map_err(repair_rejection_error)?;
    Ok(FinalNarRepairInspection {
        original_path_info,
        observed,
        transaction_plan,
        artifact_attestation,
        signature_count,
        store_dir: handle.store_dir().to_string(),
    })
}

pub async fn execute_final_nar_repair(
    handle: &StoreHandle,
    inspection: FinalNarRepairInspection,
    signing_key: &SigningKey<ed25519_dalek::SigningKey>,
) -> Result<FinalNarRepairReport, Error> {
    execute_final_nar_repair_with_profile(handle, inspection, signing_key, handle.backend().profile()).await
}

async fn execute_final_nar_repair_with_profile(
    handle: &StoreHandle,
    inspection: FinalNarRepairInspection,
    signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    profile: crate::StoreBackendCapabilityProfile,
) -> Result<FinalNarRepairReport, Error> {
    require_final_nar_repair(handle, profile)?;
    assert!(!signing_key.name().is_empty(), "signing key name must not be empty");
    assert!(!inspection.original_path_info.store_path.name().is_empty());
    handle.revalidate_overlay_bases()?;
    if inspection.transaction_plan.final_nar_plan == FinalNarRepairPlan::Current {
        return Ok(inspection.report(true));
    }
    validate_shell_transaction_plan(&inspection.transaction_plan, inspection.artifact_attestation.is_some())?;
    let current = load_exact_pathinfo(handle, &inspection.original_path_info.store_path).await?;
    if current != inspection.original_path_info {
        return Err(repair_error("PathInfo changed after inspection; rerun dry-run"));
    }

    let mut repaired = inspection.original_path_info.clone();
    repaired.nar_size = inspection.observed.nar_size;
    repaired.nar_sha256 = inspection.observed.nar_sha256;
    repaired.signatures.clear();
    repaired.signatures.push(sign_pathinfo(&repaired, signing_key, handle.store_dir()));
    let staged_attestation =
        stage_artifact_attestation(handle, inspection.artifact_attestation.as_ref(), &repaired).await?;

    persist_repaired_state(handle, RepairPersistenceInput {
        original: &inspection.original_path_info,
        repaired: &repaired,
        staged_attestation: staged_attestation.as_ref(),
    })
    .await?;

    verify_repaired_state(handle, &repaired, staged_attestation.as_ref()).await?;
    handle.revalidate_overlay_bases()?;
    let decision = plan_repair_report(RepairReportRequest {
        final_nar_plan: inspection.transaction_plan.final_nar_plan,
        artifact_attestation_present: staged_attestation.is_some(),
        execution_mode: RepairExecutionMode::Execute,
        execution_completed: true,
    });
    Ok(repair_report(RepairReportInput {
        original: &inspection.original_path_info,
        store_dir: handle.store_dir(),
        observed: inspection.observed,
        status: shell_report_status(decision.status),
        is_execution_requested: decision.execution_requested,
        is_mutated: decision.mutated,
        old_signature_count: inspection.signature_count,
        signer: Some(signing_key.name().to_string()),
        artifact_attestation: shell_sidecar_status(decision.sidecar_disposition, true),
    }))
}

fn validate_shell_transaction_plan(
    plan: &RepairTransactionPlan,
    artifact_attestation_present: bool,
) -> Result<(), Error> {
    debug_assert_eq!(plan.final_nar_plan, FinalNarRepairPlan::Repair);
    debug_assert!(!plan.mutation_intents.is_empty());
    let has_valid_intents = matches!(
        (artifact_attestation_present, plan.mutation_intents.as_slice()),
        (true, [
            RepairMutationIntent::PersistPathInfo,
            RepairMutationIntent::PublishArtifactAttestation,
            RepairMutationIntent::VerifyPathInfo,
            RepairMutationIntent::VerifyArtifactAttestation,
        ],) | (false, [
            RepairMutationIntent::PersistPathInfo,
            RepairMutationIntent::VerifyPathInfo,
        ],)
    );
    let has_valid_rollback = matches!(
        (artifact_attestation_present, plan.rollback_intents.as_slice()),
        (true, [
            RepairRollbackIntent::RestorePathInfo,
            RepairRollbackIntent::RemoveStagedArtifactAttestation,
        ],) | (false, [RepairRollbackIntent::RestorePathInfo])
    );
    if !has_valid_intents || !has_valid_rollback {
        return Err(repair_error("repair core returned an unsupported transaction order"));
    }
    Ok(())
}

struct RepairPersistenceInput<'a> {
    original: &'a PathInfo,
    repaired: &'a PathInfo,
    staged_attestation: Option<&'a StagedArtifactAttestation>,
}

async fn persist_repaired_state(handle: &StoreHandle, input: RepairPersistenceInput<'_>) -> Result<(), Error> {
    assert_eq!(input.original.store_path, input.repaired.store_path);
    let is_final_nar_changed =
        input.original.nar_size != input.repaired.nar_size || input.original.nar_sha256 != input.repaired.nar_sha256;
    assert!(is_final_nar_changed, "repair persistence requires changed final NAR facts");
    if let Err(error) = handle.pathinfo_service().put(input.repaired.clone()).await {
        if let Some(staged) = input.staged_attestation {
            cleanup_repair_temp(&staged.temp_path).await;
        }
        let rollback = handle.pathinfo_service().put(input.original.clone()).await;
        return rollback_result("persisting repaired PathInfo", error, rollback);
    }
    if let Some(staged) = input.staged_attestation
        && let Err(error) = tokio::fs::rename(&staged.temp_path, &staged.final_path).await
    {
        let rollback = handle.pathinfo_service().put(input.original.clone()).await;
        cleanup_repair_temp(&staged.temp_path).await;
        return rollback_result("publishing repaired artifact attestation", error, rollback);
    }
    Ok(())
}

fn rollback_result(
    phase: &str,
    error: impl std::fmt::Display,
    rollback: Result<PathInfo, snix_store::pathinfoservice::Error>,
) -> Result<(), Error> {
    assert!(!phase.is_empty(), "rollback phase must not be empty");
    assert!(!phase.ends_with(' '), "rollback phase must not end with whitespace");
    match rollback {
        Ok(_) => Err(repair_error(format!("{phase}: {error}; PathInfo rollback restored prior state"))),
        Err(rollback_error) => {
            Err(repair_error(format!("{phase}: {error}; PathInfo rollback failed: {rollback_error}")))
        }
    }
}

struct ExactStorePathInput<'a> {
    logical_store_path: &'a str,
    store_dir: &'a str,
}

fn parse_exact_store_path(input: ExactStorePathInput<'_>) -> Result<StorePath<String>, Error> {
    assert!(input.store_dir.starts_with('/'), "logical store prefix must be absolute");
    assert!(!input.store_dir.ends_with('/'), "logical store prefix must not have a trailing slash");
    if input.logical_store_path.is_empty() {
        return Err(repair_error("logical store path must not be empty"));
    }
    let store_path = StorePath::from_absolute_path_with_prefix(input.logical_store_path.as_bytes(), input.store_dir)
        .map_err(|_| {
            repair_error(format!(
                "expected exact logical store path under {}: {}",
                input.store_dir, input.logical_store_path
            ))
        })?;
    if store_path.to_absolute_path_with_prefix(input.store_dir) != input.logical_store_path {
        return Err(repair_error(format!("path is not canonical and exact: {}", input.logical_store_path)));
    }
    Ok(store_path)
}

async fn load_exact_pathinfo(handle: &StoreHandle, store_path: &StorePath<String>) -> Result<PathInfo, Error> {
    let loaded = handle
        .pathinfo_service()
        .get(*store_path.digest())
        .await
        .map_err(|error| repair_error(format!("loading PathInfo: {error}")))?;
    let Some(path_info) = loaded else {
        return Err(repair_error(format!("PathInfo is missing for {store_path}")));
    };
    if path_info.store_path != *store_path {
        return Err(repair_error(format!(
            "PathInfo digest collision: requested {store_path}, stored {}",
            path_info.store_path
        )));
    }
    Ok(path_info)
}

async fn calculate_final_nar_facts(handle: &StoreHandle, path_info: &PathInfo) -> Result<FinalNarFacts, Error> {
    let renderer = SimpleRenderer::new(handle.blob_service(), handle.directory_service());
    let (nar_size, nar_sha256) = renderer
        .calculate_nar(&path_info.node)
        .await
        .map_err(|error| repair_error(format!("rendering final NAR for {}: {error}", path_info.store_path)))?;
    if nar_size == 0 {
        return Err(repair_error("observed final NAR size is zero"));
    }
    Ok(FinalNarFacts { nar_size, nar_sha256 })
}

fn path_info_final_nar_facts(path_info: &PathInfo) -> FinalNarFacts {
    FinalNarFacts {
        nar_size: path_info.nar_size,
        nar_sha256: path_info.nar_sha256,
    }
}

fn validate_artifact_attestation(
    attestation: Option<&ArtifactAttestation>,
    path_info: &PathInfo,
    store_dir: &str,
) -> bool {
    let Some(attestation) = attestation else {
        return true;
    };
    let expected_path = path_info.store_path.to_absolute_path_with_prefix(store_dir);
    let expected_digest = nar_sha256_digest(&path_info.nar_sha256);
    attestation.facts.logical_path == expected_path && attestation.facts.content_digest == expected_digest
}

fn sign_pathinfo(
    path_info: &PathInfo,
    signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    store_dir: &str,
) -> Signature<String> {
    let store_path_ref: StorePathRef = path_info.store_path.as_ref();
    let references = path_info.references.iter().map(StorePath::as_ref).collect::<Vec<_>>();
    let fingerprint = nix_compat::narinfo::fingerprint_with_store_dir(
        &store_path_ref,
        &path_info.nar_sha256,
        path_info.nar_size,
        references.iter(),
        store_dir,
    );
    signing_key.sign(fingerprint.as_bytes()).to_owned()
}

struct StagedArtifactAttestation {
    final_path: PathBuf,
    temp_path: PathBuf,
    expected: ArtifactAttestation,
}

async fn stage_artifact_attestation(
    handle: &StoreHandle,
    existing: Option<&ArtifactAttestation>,
    repaired: &PathInfo,
) -> Result<Option<StagedArtifactAttestation>, Error> {
    assert!(!repaired.store_path.name().is_empty(), "repaired store path name must not be empty");
    assert!(handle.store_dir().starts_with('/'), "logical store prefix must be absolute");
    let Some(existing) = existing else {
        return Ok(None);
    };
    let mut expected = existing.clone();
    expected.facts.content_digest = nar_sha256_digest(&repaired.nar_sha256);
    let bytes = expected
        .canonical_bytes()
        .map_err(|error| repair_error(format!("canonicalizing repaired artifact attestation: {error}")))?;
    let final_path = artifact_attestation_file_path(handle.state_dir(), handle.store_dir(), &repaired.store_path);
    let parent = final_path
        .parent()
        .ok_or_else(|| repair_error(format!("artifact attestation path has no parent: {}", final_path.display())))?;
    tokio::fs::create_dir_all(parent).await.map_err(|error| {
        repair_error(format!("creating artifact attestation directory {}: {error}", parent.display()))
    })?;
    let temp_path = final_path.with_extension(ATTESTATION_REPAIR_TEMP_EXTENSION);
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .await
        .map_err(|error| repair_error(format!("staging artifact attestation {}: {error}", temp_path.display())))?;
    if let Err(error) = file.write_all(&bytes).await {
        drop(file);
        cleanup_repair_temp(&temp_path).await;
        return Err(repair_error(format!("writing staged artifact attestation {}: {error}", temp_path.display())));
    }
    if let Err(error) = file.sync_all().await {
        drop(file);
        cleanup_repair_temp(&temp_path).await;
        return Err(repair_error(format!("syncing staged artifact attestation {}: {error}", temp_path.display())));
    }
    Ok(Some(StagedArtifactAttestation {
        final_path,
        temp_path,
        expected,
    }))
}

async fn cleanup_repair_temp(path: &Path) {
    assert!(path.file_name().is_some(), "repair temp path must have a file name");
    assert!(path.extension().is_some(), "repair temp path must have an extension");
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => tracing::warn!(path = %path.display(), %error, "failed to clean final NAR repair temp file"),
    }
}

async fn verify_repaired_state(
    handle: &StoreHandle,
    repaired: &PathInfo,
    staged_attestation: Option<&StagedArtifactAttestation>,
) -> Result<(), Error> {
    let persisted = load_exact_pathinfo(handle, &repaired.store_path).await?;
    if persisted != *repaired {
        return Err(repair_error("post-write PathInfo verification mismatch"));
    }
    if let Some(staged) = staged_attestation {
        let persisted_attestation = handle
            .get_artifact_attestation(&repaired.store_path)
            .await?
            .ok_or_else(|| repair_error("post-write artifact attestation is missing"))?;
        if persisted_attestation.attestation != staged.expected {
            return Err(repair_error("post-write artifact attestation verification mismatch"));
        }
    }
    Ok(())
}

const fn shell_report_status(status: RepairReportStatus) -> FinalNarRepairStatus {
    match status {
        RepairReportStatus::Current => FinalNarRepairStatus::Current,
        RepairReportStatus::WouldRepair => FinalNarRepairStatus::WouldRepair,
        RepairReportStatus::Repaired => FinalNarRepairStatus::Repaired,
    }
}

const fn shell_sidecar_status(
    disposition: RepairSidecarDisposition,
    execution_completed: bool,
) -> ArtifactAttestationRepairStatus {
    match (disposition, execution_completed) {
        (RepairSidecarDisposition::Absent, _) => ArtifactAttestationRepairStatus::Absent,
        (RepairSidecarDisposition::Current, _) => ArtifactAttestationRepairStatus::Current,
        (RepairSidecarDisposition::Refresh, false) => ArtifactAttestationRepairStatus::WouldRefresh,
        (RepairSidecarDisposition::Refresh, true) => ArtifactAttestationRepairStatus::Refreshed,
    }
}

struct RepairReportInput<'a> {
    original: &'a PathInfo,
    store_dir: &'a str,
    observed: FinalNarFacts,
    status: FinalNarRepairStatus,
    is_execution_requested: bool,
    is_mutated: bool,
    old_signature_count: u32,
    signer: Option<String>,
    artifact_attestation: ArtifactAttestationRepairStatus,
}

fn repair_report(input: RepairReportInput<'_>) -> FinalNarRepairReport {
    assert!(input.old_signature_count > 0, "repair report requires prior signatures");
    assert!(input.store_dir.starts_with('/'), "logical store prefix must be absolute");
    let new_signature_count = match input.status {
        FinalNarRepairStatus::WouldRepair | FinalNarRepairStatus::Repaired => 1,
        FinalNarRepairStatus::Current => input.old_signature_count,
    };
    FinalNarRepairReport {
        format: "mantle-pathinfo-final-nar-repair-v1",
        store_path: input.original.store_path.to_absolute_path_with_prefix(input.store_dir),
        status: input.status,
        execution_requested: input.is_execution_requested,
        mutated: input.is_mutated,
        recorded_nar_size: input.original.nar_size,
        recorded_nar_sha256: data_encoding::HEXLOWER.encode(&input.original.nar_sha256),
        observed_nar_size: input.observed.nar_size,
        observed_nar_sha256: data_encoding::HEXLOWER.encode(&input.observed.nar_sha256),
        old_signature_count: input.old_signature_count,
        new_signature_count,
        signer: input.signer,
        artifact_attestation: input.artifact_attestation,
        non_claim: "This report records one local final-NAR inspection or migration; it does not recover historical signer authority or prove content correctness, provenance, reproducibility, archive compatibility, or release eligibility.",
    }
}

fn nar_sha256_digest(digest: &[u8; SHA256_DIGEST_BYTES]) -> String {
    format!("nar-sha256:{}", data_encoding::HEXLOWER.encode(digest))
}

fn repair_rejection_error(rejection: FinalNarRepairRejection) -> Error {
    let reason = match rejection {
        FinalNarRepairRejection::Unsigned => "PathInfo is unsigned; use ordinary store sign or rebuild",
        FinalNarRepairRejection::InvalidObservedNarFacts => "observed final NAR facts are invalid",
        FinalNarRepairRejection::IncompleteContent => "local castore content is missing or incomplete",
        FinalNarRepairRejection::InvalidCaPathIdentity => "CA metadata does not derive the selected store path",
        FinalNarRepairRejection::InvalidArtifactAttestation => "artifact attestation does not match the stale PathInfo",
        FinalNarRepairRejection::InvalidPathIdentity => "repair path identity is not canonical",
        FinalNarRepairRejection::IdentityEncodingOverflow => "repair plan identity encoding overflowed",
    };
    repair_error(reason)
}

fn repair_error(message: impl Into<String>) -> Error {
    Error::Store(format!("final NAR repair: {}", message.into()))
}

#[cfg(test)]
// r[verify store_transports.pathinfo_final_nar_migration]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;

    use crunch_attestation::Claims;
    use futures::stream::BoxStream;
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::NixHash;
    use nix_compat::store_path::build_ca_path_with_store_dir;
    use snix_castore::Node;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::ArtifactProvenance;
    use crate::StoreConfig;
    use crate::StoreFallbackMode;
    use crate::StoreHandleServices;
    use crate::layer::StoreLayer;
    use crate::persist_artifact_attestation;

    const STORE_DIR: &str = "/mantle/store";
    const MARKER_HASH_BYTE: u8 = 0xA5;
    const MISSING_PATH_DIGEST_BYTE: u8 = 0x19;
    const SIGNING_KEY_BYTE: u8 = 0x2A;
    const STALE_NAR_HASH_BYTE: u8 = 0x5A;
    const STORE_PATH_DIGEST_BYTES: usize = 20;
    const TEST_SERVICE_CAPACITY: usize = 32;

    struct RejectingPutPathInfoService {
        inner: LruPathInfoService,
        put_failures_remaining: AtomicU32,
    }

    impl RejectingPutPathInfoService {
        fn new() -> Self {
            Self {
                inner: LruPathInfoService::with_capacity(
                    "repair-reject-put".to_string(),
                    NonZeroUsize::new(TEST_SERVICE_CAPACITY).unwrap(),
                ),
                put_failures_remaining: AtomicU32::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl PathInfoService for RejectingPutPathInfoService {
        async fn get(
            &self,
            digest: [u8; STORE_PATH_DIGEST_BYTES],
        ) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
            self.inner.get(digest).await
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
            let did_consume_failure = self
                .put_failures_remaining
                .try_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| remaining.checked_sub(1))
                .is_ok();
            if did_consume_failure {
                return Err(std::io::Error::other("forced PathInfo persistence failure").into());
            }
            self.inner.put(path_info).await
        }

        fn list(&self) -> BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
            self.inner.list()
        }
    }

    fn planning_facts(recorded: FinalNarFacts, observed: FinalNarFacts) -> FinalNarRepairPlanningFacts {
        FinalNarRepairPlanningFacts {
            recorded,
            observed,
            signature_count: 1,
            is_content_complete: true,
            is_ca_path_identity_valid: true,
            is_attestation_valid: true,
        }
    }

    #[test]
    fn pure_plan_distinguishes_current_and_stale_facts() {
        let current = FinalNarFacts {
            nar_size: 10,
            nar_sha256: [1; SHA256_DIGEST_BYTES],
        };
        let stale = FinalNarFacts {
            nar_size: 10,
            nar_sha256: [2; SHA256_DIGEST_BYTES],
        };

        assert_eq!(plan_final_nar_repair(planning_facts(current, current)), Ok(FinalNarRepairPlan::Current));
        assert_eq!(plan_final_nar_repair(planning_facts(stale, current)), Ok(FinalNarRepairPlan::Repair));
    }

    #[test]
    fn pure_plan_rejects_each_unsafe_candidate() {
        let facts = FinalNarFacts {
            nar_size: 10,
            nar_sha256: [1; SHA256_DIGEST_BYTES],
        };
        let mut candidate = planning_facts(facts, facts);
        candidate.signature_count = 0;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::Unsigned));
        candidate = planning_facts(facts, facts);
        candidate.observed.nar_size = 0;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::InvalidObservedNarFacts));
        candidate = planning_facts(facts, facts);
        candidate.is_content_complete = false;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::IncompleteContent));
        candidate = planning_facts(facts, facts);
        candidate.is_ca_path_identity_valid = false;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::InvalidCaPathIdentity));
        candidate = planning_facts(facts, facts);
        candidate.is_attestation_valid = false;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::InvalidArtifactAttestation));
    }

    async fn open_test_store(dir: &std::path::Path) -> StoreHandle {
        StoreHandle::open(StoreConfig {
            backend: crate::StoreBackend::Snix,
            state_dir: dir.join("state"),
            output_dir: dir.join("store"),
            remote_cache_urls: Vec::new(),
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: STORE_DIR.to_string(),
            base_state_dirs: Vec::new(),
        })
        .await
        .unwrap()
    }

    fn test_store_with_rejecting_put(state_dir: &std::path::Path) -> (StoreHandle, Arc<RejectingPutPathInfoService>) {
        let blob_service = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
        let directory_service = Arc::new(
            RedbDirectoryService::new_temporary("repair-directory".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap(),
        ) as Arc<dyn DirectoryService>;
        let pathinfo_service = Arc::new(RejectingPutPathInfoService::new());
        let handle = StoreHandle::from_services_with_store_dir(
            crate::StoreBackend::Snix,
            StoreHandleServices {
                blob_service,
                directory_service,
                pathinfo_service: pathinfo_service.clone(),
                remote_pathinfo: None,
                state_dir: state_dir.to_path_buf(),
                output_dir_str: state_dir.display().to_string(),
                publishers: Vec::new(),
            },
            STORE_DIR.to_string(),
        )
        .unwrap();
        (handle, pathinfo_service)
    }

    fn signing_key() -> SigningKey<ed25519_dalek::SigningKey> {
        SigningKey::new(
            "repair-test-1".to_string(),
            ed25519_dalek::SigningKey::from_bytes(&[SIGNING_KEY_BYTE; SHA256_DIGEST_BYTES]),
        )
    }

    async fn stale_signed_pathinfo(handle: &StoreHandle, name: &str) -> PathInfo {
        let content = b"final rewritten output";
        let mut writer = handle.blob_service().open_write().await;
        writer.write_all(content).await.unwrap();
        let digest = writer.close().await.unwrap();
        let node = Node::File {
            digest,
            size: u64::try_from(content.len()).unwrap(),
            executable: false,
        };
        let ca = CAHash::Nar(NixHash::Sha256([MARKER_HASH_BYTE; SHA256_DIGEST_BYTES]));
        let store_path = build_ca_path_with_store_dir(name, &ca, Vec::<String>::new(), false, STORE_DIR).unwrap();
        let mut path_info = PathInfo {
            store_path,
            node,
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [STALE_NAR_HASH_BYTE; SHA256_DIGEST_BYTES],
            signatures: Vec::new(),
            deriver: None,
            ca: Some(ca),
        };
        path_info.signatures.push(sign_pathinfo(&path_info, &signing_key(), STORE_DIR));
        path_info
    }

    #[tokio::test]
    async fn dry_run_does_not_mutate_stale_pathinfo_or_attestation() {
        let dir = tempfile::tempdir().unwrap();
        let store = open_test_store(dir.path()).await;
        let path_info = stale_signed_pathinfo(&store, "dry-run").await;
        store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let stored_attestation = persist_artifact_attestation(
            store.state_dir(),
            STORE_DIR,
            &path_info,
            "out",
            Some(&ArtifactProvenance {
                claims: Some(Claims {
                    supplier: Some("Historical Supplier".to_string()),
                    ..Default::default()
                }),
                input_sources: Vec::new(),
                input_artifacts: Vec::new(),
                store_layer: StoreLayer::Overlay,
            }),
        )
        .await
        .unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);

        let inspection = inspect_final_nar_repair(&store, &logical_path).await.unwrap();
        let report = inspection.report(false);
        let persisted = store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().unwrap();
        let persisted_attestation = store.get_artifact_attestation(&path_info.store_path).await.unwrap().unwrap();

        assert_eq!(report.status, FinalNarRepairStatus::WouldRepair);
        assert!(!report.execution_requested);
        assert!(!report.mutated);
        assert_eq!(persisted, path_info);
        assert_eq!(persisted_attestation, stored_attestation);
    }

    #[tokio::test]
    async fn current_pathinfo_is_an_idempotent_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let store = open_test_store(dir.path()).await;
        let mut path_info = stale_signed_pathinfo(&store, "current").await;
        let observed = calculate_final_nar_facts(&store, &path_info).await.unwrap();
        path_info.nar_size = observed.nar_size;
        path_info.nar_sha256 = observed.nar_sha256;
        path_info.signatures.clear();
        path_info.signatures.push(sign_pathinfo(&path_info, &signing_key(), STORE_DIR));
        store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let old_attestation =
            persist_artifact_attestation(store.state_dir(), STORE_DIR, &path_info, "out", None).await.unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);
        let inspection = inspect_final_nar_repair(&store, &logical_path).await.unwrap();

        let report = execute_final_nar_repair(&store, inspection, &signing_key()).await.unwrap();
        let persisted = store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().unwrap();
        let persisted_attestation = store.get_artifact_attestation(&path_info.store_path).await.unwrap().unwrap();

        assert_eq!(report.status, FinalNarRepairStatus::Current);
        assert!(report.execution_requested);
        assert!(!report.mutated);
        assert_eq!(persisted, path_info);
        assert_eq!(persisted_attestation, old_attestation);
    }

    #[tokio::test]
    async fn execute_repairs_facts_replaces_signatures_and_preserves_attestation_graph() {
        let dir = tempfile::tempdir().unwrap();
        let store = open_test_store(dir.path()).await;
        let path_info = stale_signed_pathinfo(&store, "execute").await;
        store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let old_attestation = persist_artifact_attestation(
            store.state_dir(),
            STORE_DIR,
            &path_info,
            "out",
            Some(&ArtifactProvenance {
                claims: Some(Claims {
                    supplier: Some("Historical Supplier".to_string()),
                    ..Default::default()
                }),
                input_sources: Vec::new(),
                input_artifacts: Vec::new(),
                store_layer: StoreLayer::Overlay,
            }),
        )
        .await
        .unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);
        let inspection = inspect_final_nar_repair(&store, &logical_path).await.unwrap();

        let report = execute_final_nar_repair(&store, inspection, &signing_key()).await.unwrap();
        let repaired = store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().unwrap();
        let repaired_attestation = store.get_artifact_attestation(&path_info.store_path).await.unwrap().unwrap();

        assert_eq!(report.status, FinalNarRepairStatus::Repaired);
        assert!(report.execution_requested);
        assert!(report.mutated);
        assert_ne!(repaired.nar_sha256, path_info.nar_sha256);
        assert_eq!(repaired.signatures.len(), 1);
        assert_eq!(repaired.node, path_info.node);
        assert_eq!(repaired.ca, path_info.ca);
        assert_eq!(repaired_attestation.attestation.claims, old_attestation.attestation.claims);
        assert_eq!(repaired_attestation.attestation.nodes, old_attestation.attestation.nodes);
        assert_eq!(repaired_attestation.attestation.edges, old_attestation.attestation.edges);
        assert_ne!(
            repaired_attestation.attestation.facts.content_digest,
            old_attestation.attestation.facts.content_digest
        );
    }

    #[tokio::test]
    async fn missing_exact_pathinfo_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = open_test_store(dir.path()).await;
        let missing: StorePath<String> =
            StorePath::from_name_and_digest_fixed("missing", [MISSING_PATH_DIGEST_BYTE; STORE_PATH_DIGEST_BYTES])
                .unwrap();
        let logical_path = missing.to_absolute_path_with_prefix(STORE_DIR);

        let error = inspect_final_nar_repair(&store, &logical_path).await.unwrap_err();
        let persisted = store.pathinfo_service().get(*missing.digest()).await.unwrap();

        assert!(error.to_string().contains("PathInfo is missing"));
        assert!(persisted.is_none());
    }

    #[tokio::test]
    async fn unsigned_stale_pathinfo_is_rejected_without_mutation() {
        let dir = tempfile::tempdir().unwrap();
        let store = open_test_store(dir.path()).await;
        let mut path_info = stale_signed_pathinfo(&store, "unsigned").await;
        path_info.signatures.clear();
        store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);

        let error = inspect_final_nar_repair(&store, &logical_path).await.unwrap_err();
        let persisted = store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().unwrap();

        assert!(error.to_string().contains("unsigned"));
        assert_eq!(persisted, path_info);
    }

    #[tokio::test]
    async fn pathinfo_persistence_failure_is_reported_without_mutation() {
        let dir = tempfile::tempdir().unwrap();
        let (store, service) = test_store_with_rejecting_put(dir.path());
        let path_info = stale_signed_pathinfo(&store, "put-failure").await;
        service.put(path_info.clone()).await.unwrap();
        let old_attestation =
            persist_artifact_attestation(store.state_dir(), STORE_DIR, &path_info, "out", None).await.unwrap();
        let attestation_path = artifact_attestation_file_path(store.state_dir(), STORE_DIR, &path_info.store_path);
        let temp_path = attestation_path.with_extension(ATTESTATION_REPAIR_TEMP_EXTENSION);
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);
        let inspection = inspect_final_nar_repair(&store, &logical_path).await.unwrap();
        service.put_failures_remaining.store(1, Ordering::SeqCst);

        let error = execute_final_nar_repair(&store, inspection, &signing_key()).await.unwrap_err();
        let persisted = service.get(*path_info.store_path.digest()).await.unwrap().unwrap();
        let persisted_attestation = store.get_artifact_attestation(&path_info.store_path).await.unwrap().unwrap();

        assert!(error.to_string().contains("persisting repaired PathInfo"));
        assert!(error.to_string().contains("rollback restored prior state"));
        assert_eq!(persisted, path_info);
        assert_eq!(persisted_attestation, old_attestation);
        assert!(!temp_path.exists());
    }

    #[tokio::test]
    async fn invalid_ca_identity_is_rejected_without_pathinfo_mutation() {
        const WRONG_CA_HASH_BYTE: u8 = 0x3C;
        let dir = tempfile::tempdir().unwrap();
        let store = open_test_store(dir.path()).await;
        let mut path_info = stale_signed_pathinfo(&store, "invalid-ca").await;
        path_info.ca = Some(CAHash::Nar(NixHash::Sha256([WRONG_CA_HASH_BYTE; SHA256_DIGEST_BYTES])));
        path_info.signatures.clear();
        path_info.signatures.push(sign_pathinfo(&path_info, &signing_key(), STORE_DIR));
        store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);

        let error = inspect_final_nar_repair(&store, &logical_path).await.unwrap_err();
        let persisted = store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().unwrap();

        assert!(error.to_string().contains("CA metadata"));
        assert_eq!(persisted, path_info);
    }

    #[tokio::test]
    async fn stale_artifact_attestation_is_rejected_without_pathinfo_mutation() {
        const OTHER_NAR_HASH_BYTE: u8 = 0x6D;
        let dir = tempfile::tempdir().unwrap();
        let store = open_test_store(dir.path()).await;
        let path_info = stale_signed_pathinfo(&store, "stale-attestation").await;
        store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let mut attested_path_info = path_info.clone();
        attested_path_info.nar_sha256 = [OTHER_NAR_HASH_BYTE; SHA256_DIGEST_BYTES];
        persist_artifact_attestation(store.state_dir(), STORE_DIR, &attested_path_info, "out", None)
            .await
            .unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);

        let error = inspect_final_nar_repair(&store, &logical_path).await.unwrap_err();
        let persisted = store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().unwrap();

        assert!(error.to_string().contains("artifact attestation"));
        assert_eq!(persisted, path_info);
    }

    #[tokio::test]
    async fn incomplete_content_is_rejected_without_pathinfo_mutation() {
        let dir = tempfile::tempdir().unwrap();
        let store = open_test_store(dir.path()).await;
        let mut path_info = stale_signed_pathinfo(&store, "incomplete").await;
        if let Node::File { digest, .. } = &mut path_info.node {
            *digest = blake3::hash(b"missing").into();
        }
        path_info.signatures.clear();
        path_info.signatures.push(sign_pathinfo(&path_info, &signing_key(), STORE_DIR));
        store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);

        let error = inspect_final_nar_repair(&store, &logical_path).await.unwrap_err();
        let persisted = store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().unwrap();

        assert!(error.to_string().contains("missing or incomplete"));
        assert_eq!(persisted, path_info);
    }

    fn snapshot_repair_tree(root: &Path) -> std::collections::BTreeMap<PathBuf, Option<Vec<u8>>> {
        let mut entries = std::collections::BTreeMap::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(directory).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                let kind = entry.file_type().unwrap();
                if kind.is_dir() {
                    entries.insert(relative, None);
                    pending.push(path);
                } else {
                    assert!(kind.is_file(), "unexpected non-file in repair state: {}", path.display());
                    entries.insert(relative, Some(std::fs::read(path).unwrap()));
                }
            }
        }
        entries
    }

    #[tokio::test]
    async fn casita_repair_library_entrypoints_reject_without_publishing() {
        let dir = tempfile::tempdir().unwrap();
        let snix = open_test_store(dir.path()).await;
        let path_info = stale_signed_pathinfo(&snix, "casita-rejected").await;
        snix.pathinfo_service().put(path_info.clone()).await.unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);
        let inspection = inspect_final_nar_repair(&snix, &logical_path).await.unwrap();
        let casita_state = dir.path().join("casita-state");
        let casita_output = dir.path().join("casita-store");
        std::fs::create_dir_all(&casita_output).unwrap();
        let casita = StoreHandle::open(StoreConfig::new(
            crate::StoreBackend::Casita,
            casita_state.clone(),
            casita_output.clone(),
            STORE_DIR.to_string(),
        ))
        .await
        .unwrap();
        std::fs::write(casita_state.join("keep.txt"), b"preserve Casita state").unwrap();
        std::fs::write(casita_output.join("keep.txt"), b"preserve output state").unwrap();
        assert!(casita.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().is_none());

        let state_before = snapshot_repair_tree(&casita_state);
        let output_before = snapshot_repair_tree(&casita_output);
        let inspect_error = inspect_final_nar_repair(&casita, &logical_path).await.unwrap_err();
        assert_eq!(snapshot_repair_tree(&casita_state), state_before);
        assert_eq!(snapshot_repair_tree(&casita_output), output_before);
        let execute_error = execute_final_nar_repair(&casita, inspection, &signing_key()).await.unwrap_err();
        assert_eq!(snapshot_repair_tree(&casita_state), state_before);
        assert_eq!(snapshot_repair_tree(&casita_output), output_before);

        assert!(inspect_error.to_string().contains("casita-repair-final-nar-unsupported"));
        assert!(execute_error.to_string().contains("casita-repair-final-nar-unsupported"));
        assert!(casita.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().is_none());
        assert_eq!(snix.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap(), Some(path_info));
    }

    #[tokio::test]
    async fn undeclared_repair_capability_rejects_snix_library_entrypoints_before_effects() {
        let dir = tempfile::tempdir().unwrap();
        let snix = open_test_store(dir.path()).await;
        let path_info = stale_signed_pathinfo(&snix, "profile-rejected").await;
        snix.pathinfo_service().put(path_info.clone()).await.unwrap();
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(STORE_DIR);
        let inspection = inspect_final_nar_repair(&snix, &logical_path).await.unwrap();

        let mut profile = crate::StoreBackend::Snix.profile();
        profile.core = crate::StoreBackend::Casita.profile().core;
        let state_dir = dir.path().join("state");
        let output_dir = dir.path().join("store");
        std::fs::create_dir_all(&output_dir).unwrap();
        std::fs::write(state_dir.join("keep.txt"), b"preserve Snix state").unwrap();
        std::fs::write(output_dir.join("keep.txt"), b"preserve output state").unwrap();
        let state_before = snapshot_repair_tree(&state_dir);
        let output_before = snapshot_repair_tree(&output_dir);

        let inspect_error =
            inspect_final_nar_repair_with_profile(&snix, "/not/a/valid/store/path", profile).await.unwrap_err();
        assert!(inspect_error.to_string().contains("snix-repair-final-nar-unsupported"));
        assert_eq!(snapshot_repair_tree(&state_dir), state_before);
        assert_eq!(snapshot_repair_tree(&output_dir), output_before);

        let execute_error =
            execute_final_nar_repair_with_profile(&snix, inspection, &signing_key(), profile).await.unwrap_err();
        assert!(execute_error.to_string().contains("snix-repair-final-nar-unsupported"));
        assert_eq!(snapshot_repair_tree(&state_dir), state_before);
        assert_eq!(snapshot_repair_tree(&output_dir), output_before);
        assert_eq!(snix.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap(), Some(path_info));
    }
}
