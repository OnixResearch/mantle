#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::BoundedComponentClaim;
use crunch_wasm_component_core::ComponentArtifactAttestation;
use crunch_wasm_component_core::ComponentBuildReport;
use crunch_wasm_component_core::ComponentEvidenceRequest;
use crunch_wasm_component_core::ComponentReleaseBinding;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::ComponentStageStatus;
use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::StageReportInput;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::build_component_evidence;
use crunch_wasm_component_core::build_component_report;
use crunch_wasm_component_core::stage_report_identity;
use serde::Serialize;

use crate::Error;
use crate::OctetValidationEvidence;
use crate::PIPELINE_EXECUTION_REPORT_SCHEMA;
use crate::PipelineArtifact;
use crate::PipelineBlocker;
use crate::PipelineExecutionReport;
use crate::ToolExecutionReceipt;
use crate::write_json_new;

const EXECUTION_REPORT_FILE: &str = "execution-report.json";
const MAX_EXECUTION_RECEIPTS: usize = 128;
const MAX_EXECUTION_ARTIFACTS: usize = 128;
const MAX_EXECUTION_BLOCKERS: usize = 128;
const MAX_EXECUTION_STAGES: usize = 64;
const MAX_EXECUTION_NON_CLAIMS: usize = 32;
const MAX_BLOCKER_MESSAGE_BYTES: usize = 16 * 1024;
const BOUNDED_BLOCKER_MESSAGE: &str = "blocker message exceeded the execution-report byte bound";

#[derive(Debug)]
pub(crate) struct ExecutionState {
    pub request_blake3: Blake3Identity,
    pub cohort_blake3: Option<Blake3Identity>,
    pub receipts: Vec<ToolExecutionReceipt>,
    pub artifacts: Vec<PipelineArtifact>,
    pub blockers: Vec<PipelineBlocker>,
    pub octet_validations: Vec<OctetValidationEvidence>,
    pub stage_inputs: Vec<StageReportInput>,
    pub last_stage_identity: Option<Blake3Identity>,
    pub materialization_bundle: Option<MaterializationBundle>,
    pub materialization_bundle_object: Option<StoreObject>,
    pub non_claims: Vec<String>,
}

pub(crate) struct StageEvidence {
    pub artifact: Option<StoreObject>,
    pub tool_identity: Option<Blake3Identity>,
    pub profile_identity: Option<Blake3Identity>,
    pub claims: Vec<BoundedComponentClaim>,
}

impl StageEvidence {
    pub(crate) fn new(
        artifact: Option<StoreObject>,
        tool_identity: Option<Blake3Identity>,
        profile_identity: Option<Blake3Identity>,
        claims: Vec<BoundedComponentClaim>,
    ) -> Self {
        Self {
            artifact,
            tool_identity,
            profile_identity,
            claims,
        }
    }
}

#[derive(Serialize)]
struct ExecutionReportIdentityInput {
    schema: String,
    request_blake3: Blake3Identity,
    cohort_blake3: Option<Blake3Identity>,
    stage_receipts: Vec<ToolExecutionReceipt>,
    artifacts: Vec<PipelineArtifact>,
    blockers: Vec<PipelineBlocker>,
    octet_validations: Vec<OctetValidationEvidence>,
    component_report: Option<ComponentBuildReport>,
    materialization_bundle: Option<MaterializationBundle>,
    component_attestation: Option<ComponentArtifactAttestation>,
    release_binding: Option<ComponentReleaseBinding>,
    final_status: String,
    non_claims: Vec<String>,
}

impl ExecutionState {
    pub fn new(request_blake3: Blake3Identity, non_claims: Vec<String>) -> Result<Self, Error> {
        if non_claims.is_empty() || non_claims.len() > MAX_EXECUTION_NON_CLAIMS {
            return Err(Error::Invalid(format!("execution report requires 1..={MAX_EXECUTION_NON_CLAIMS} non-claims")));
        }
        debug_assert!(!request_blake3.clone().into_hex().is_empty());
        debug_assert!(non_claims.len() <= MAX_EXECUTION_NON_CLAIMS);
        Ok(Self {
            request_blake3,
            cohort_blake3: None,
            receipts: Vec::new(),
            artifacts: Vec::new(),
            blockers: Vec::new(),
            octet_validations: Vec::new(),
            stage_inputs: Vec::new(),
            last_stage_identity: None,
            materialization_bundle: None,
            materialization_bundle_object: None,
            non_claims,
        })
    }

    pub fn block(&mut self, code: &str, stage_key: &str, message: impl Into<String>) {
        if self.blockers.len() >= MAX_EXECUTION_BLOCKERS {
            return;
        }
        let message = message.into();
        let message = if message.len() > MAX_BLOCKER_MESSAGE_BYTES {
            BOUNDED_BLOCKER_MESSAGE.to_string()
        } else {
            message
        };
        self.blockers.push(PipelineBlocker {
            code: code.to_string(),
            stage_key: stage_key.to_string(),
            message,
        });
        debug_assert!(self.blockers.len() <= MAX_EXECUTION_BLOCKERS);
        debug_assert!(self.blockers.last().is_some_and(|blocker| !blocker.message.is_empty()));
    }

    pub fn add_receipt(&mut self, receipt: ToolExecutionReceipt) -> Result<(), Error> {
        if self.receipts.len() >= MAX_EXECUTION_RECEIPTS {
            return Err(Error::Invalid(format!("execution report exceeds {MAX_EXECUTION_RECEIPTS} tool receipts")));
        }
        self.receipts.push(receipt);
        debug_assert!(self.receipts.len() <= MAX_EXECUTION_RECEIPTS);
        debug_assert!(self.receipts.last().is_some_and(|receipt| !receipt.stage_key.is_empty()));
        Ok(())
    }

    pub fn add_octet_validation(&mut self, evidence: OctetValidationEvidence) -> Result<(), Error> {
        if self.octet_validations.len() >= MAX_EXECUTION_STAGES {
            return Err(Error::Invalid(format!("execution report exceeds {MAX_EXECUTION_STAGES} Octet validations")));
        }
        self.octet_validations.push(evidence);
        debug_assert!(self.octet_validations.len() <= MAX_EXECUTION_STAGES);
        debug_assert!(self.octet_validations.last().is_some_and(|item| item.decision == "passed"));
        Ok(())
    }

    pub fn set_materialization_bundle(
        &mut self,
        bundle: MaterializationBundle,
        object: StoreObject,
    ) -> Result<(), Error> {
        if self.materialization_bundle.is_some() || self.materialization_bundle_object.is_some() {
            return Err(Error::Invalid("materialization bundle was already recorded".to_string()));
        }
        self.materialization_bundle = Some(bundle);
        self.materialization_bundle_object = Some(object);
        debug_assert!(self.materialization_bundle.is_some());
        debug_assert!(self.materialization_bundle_object.is_some());
        Ok(())
    }

    pub fn add_artifact(&mut self, role: &str, object: &StoreObject) -> Result<(), Error> {
        if self.artifacts.len() >= MAX_EXECUTION_ARTIFACTS {
            return Err(Error::Invalid(format!("execution report exceeds {MAX_EXECUTION_ARTIFACTS} artifacts")));
        }
        self.artifacts.push(PipelineArtifact {
            role: role.to_string(),
            path: object.logical_path.clone(),
            digest_blake3: object.digest_blake3.clone(),
            size_bytes: object.size_bytes,
        });
        debug_assert!(object.size_bytes > 0);
        debug_assert!(object.logical_path.starts_with('/'));
        Ok(())
    }

    pub fn push_stage(
        &mut self,
        stage_key: &str,
        kind: ComponentStageKind,
        status: ComponentStageStatus,
        evidence: StageEvidence,
    ) -> Result<(), Error> {
        if self.stage_inputs.len() >= MAX_EXECUTION_STAGES {
            return Err(Error::Invalid(format!("execution report exceeds {MAX_EXECUTION_STAGES} stages")));
        }
        let parents = self.last_stage_identity.iter().cloned().collect();
        let input = StageReportInput {
            stage_key: stage_key.to_string(),
            kind,
            status,
            parents,
            artifact: evidence.artifact,
            tool_identity_blake3: evidence.tool_identity,
            profile_identity_blake3: evidence.profile_identity,
            claims: evidence.claims,
            non_claims: self.non_claims.clone(),
        };
        let identity = stage_report_identity(input.clone())
            .map_err(|error| Error::Invalid(format!("identifying component stage `{stage_key}`: {error}")))?;
        self.stage_inputs.push(input);
        self.last_stage_identity = Some(identity);
        debug_assert!(self.stage_inputs.len() <= MAX_EXECUTION_STAGES);
        debug_assert!(self.last_stage_identity.is_some());
        Ok(())
    }
}

pub(crate) fn finish_execution(
    mut state: ExecutionState,
    scratch_root: PathBuf,
    evidence_dir: &Path,
) -> Result<PipelineExecutionReport, Error> {
    validate_state_bounds(&state)?;
    let stage_inputs = std::mem::take(&mut state.stage_inputs);
    let component_result = build_component_report(stage_inputs, state.non_claims.clone());
    for blocker in component_result.blockers {
        state.block(&blocker.code, "component-report", blocker.message);
    }
    validate_state_bounds(&state)?;
    let component_build_evidence = component_result.report;
    let (component_attestation, release_binding) = build_and_publish_component_evidence(
        &mut state,
        component_build_evidence.as_ref(),
        &scratch_root,
        evidence_dir,
    )?;
    let final_status = if state.blockers.is_empty() {
        "succeeded"
    } else {
        "blocked"
    }
    .to_string();
    let input = ExecutionReportIdentityInput {
        schema: PIPELINE_EXECUTION_REPORT_SCHEMA.to_string(),
        request_blake3: state.request_blake3.clone(),
        cohort_blake3: state.cohort_blake3.clone(),
        stage_receipts: state.receipts.clone(),
        artifacts: state.artifacts.clone(),
        blockers: state.blockers.clone(),
        octet_validations: state.octet_validations.clone(),
        component_report: component_build_evidence.clone(),
        materialization_bundle: state.materialization_bundle.clone(),
        component_attestation: component_attestation.clone(),
        release_binding: release_binding.clone(),
        final_status: final_status.clone(),
        non_claims: state.non_claims.clone(),
    };
    let canonical = serde_json::to_vec(&input)
        .map_err(|error| Error::Invalid(format!("serializing execution report identity input: {error}")))?;
    let pipeline_evidence = PipelineExecutionReport {
        schema: PIPELINE_EXECUTION_REPORT_SCHEMA.to_string(),
        request_blake3: state.request_blake3,
        cohort_blake3: state.cohort_blake3,
        stage_receipts: state.receipts,
        artifacts: state.artifacts,
        blockers: state.blockers,
        octet_validations: state.octet_validations,
        component_report: component_build_evidence,
        materialization_bundle: state.materialization_bundle,
        component_attestation,
        release_binding,
        final_status,
        report_blake3: Blake3Identity::from_slice(&canonical),
        non_claims: state.non_claims,
    };
    write_json_new(&scratch_root.join(EXECUTION_REPORT_FILE), &pipeline_evidence)?;
    publish_scratch(scratch_root, evidence_dir)?;
    debug_assert_eq!(pipeline_evidence.final_status == "succeeded", pipeline_evidence.blockers.is_empty());
    debug_assert!(evidence_dir.join(EXECUTION_REPORT_FILE).is_file());
    Ok(pipeline_evidence)
}

fn build_and_publish_component_evidence(
    state: &mut ExecutionState,
    report: Option<&ComponentBuildReport>,
    scratch_root: &Path,
    evidence_dir: &Path,
) -> Result<(Option<ComponentArtifactAttestation>, Option<ComponentReleaseBinding>), Error> {
    let Some(bundle) = state.materialization_bundle.clone() else {
        if state.blockers.is_empty() {
            state.block(
                "missing-materialization-bundle",
                "component-evidence",
                "successful pipeline omitted its bundle",
            );
        }
        return Ok((None, None));
    };
    let Some(bundle_object) = state.materialization_bundle_object.clone() else {
        state.block("missing-materialization-bundle-object", "component-evidence", "bundle object was not recorded");
        return Ok((None, None));
    };
    let Some(component_report) = report.cloned() else {
        state.block(
            "missing-component-report",
            "component-evidence",
            "component stage graph report was not constructed",
        );
        return Ok((None, None));
    };
    let Some(octet) = state.octet_validations.last().cloned() else {
        state.block("missing-octet-evidence", "component-evidence", "materialized component omitted Octet evidence");
        return Ok((None, None));
    };
    let result = build_component_evidence(ComponentEvidenceRequest {
        bundle,
        bundle_object,
        report: component_report,
        octet_profile_blake3: octet.profile_blake3,
        octet_cohort_blake3: octet.cohort_blake3,
        octet_report: octet.receipt,
    });
    for blocker in result.blockers {
        state.block(&blocker.code, "component-evidence", blocker.message);
    }
    let (Some(attestation), Some(release)) = (result.artifact_attestation, result.release_binding) else {
        return Ok((None, None));
    };
    let attestation_object =
        write_evidence_object(scratch_root, evidence_dir, "component-attestation.json", &attestation)?;
    let release_object = write_evidence_object(scratch_root, evidence_dir, "component-release-binding.json", &release)?;
    state.add_artifact("component-artifact-attestation", &attestation_object)?;
    state.add_artifact("component-release-binding", &release_object)?;
    debug_assert!(!release.release_eligible);
    debug_assert_eq!(release.artifact_attestation_blake3, attestation.attestation_blake3);
    Ok((Some(attestation), Some(release)))
}

fn write_evidence_object(
    scratch_root: &Path,
    evidence_dir: &Path,
    name: &str,
    value: &impl Serialize,
) -> Result<StoreObject, Error> {
    let scratch_path = scratch_root.join(name);
    write_json_new(&scratch_path, value)?;
    let metadata = fs::symlink_metadata(&scratch_path)
        .map_err(|error| Error::io("reading component evidence metadata", &scratch_path, error))?;
    if !metadata.file_type().is_file() || metadata.len() == 0 {
        return Err(Error::Invalid(format!("component evidence file is empty or not regular: {name}")));
    }
    let object = StoreObject {
        logical_path: evidence_dir.join(name).display().to_string(),
        digest_blake3: crate::toolchain::hash_file_bounded(&scratch_path)?,
        size_bytes: metadata.len(),
    };
    debug_assert!(object.logical_path.starts_with('/'));
    debug_assert!(object.size_bytes > 0);
    Ok(object)
}

fn validate_state_bounds(state: &ExecutionState) -> Result<(), Error> {
    if state.receipts.len() > MAX_EXECUTION_RECEIPTS {
        return Err(evidence_bound_error());
    }
    if state.artifacts.len() > MAX_EXECUTION_ARTIFACTS {
        return Err(evidence_bound_error());
    }
    if state.blockers.len() > MAX_EXECUTION_BLOCKERS {
        return Err(evidence_bound_error());
    }
    if state.octet_validations.len() > MAX_EXECUTION_STAGES || state.stage_inputs.len() > MAX_EXECUTION_STAGES {
        return Err(evidence_bound_error());
    }
    if state.non_claims.len() > MAX_EXECUTION_NON_CLAIMS {
        return Err(evidence_bound_error());
    }
    if state.blockers.iter().any(|blocker| blocker.message.len() > MAX_BLOCKER_MESSAGE_BYTES) {
        return Err(evidence_bound_error());
    }
    debug_assert!(state.blockers.iter().all(|blocker| blocker.message.len() <= MAX_BLOCKER_MESSAGE_BYTES));
    debug_assert!(state.receipts.len() <= MAX_EXECUTION_RECEIPTS);
    Ok(())
}

fn evidence_bound_error() -> Error {
    Error::Invalid("component execution evidence exceeds a fixed report bound".to_string())
}

#[cfg(target_os = "linux")]
fn publish_scratch(scratch_root: PathBuf, evidence_dir: &Path) -> Result<(), Error> {
    let source_parent = scratch_root
        .parent()
        .ok_or_else(|| Error::Invalid(format!("component scratch has no parent: {}", scratch_root.display())))?;
    let destination_parent = evidence_dir
        .parent()
        .ok_or_else(|| Error::Invalid(format!("component output has no parent: {}", evidence_dir.display())))?;
    if source_parent != destination_parent {
        return Err(Error::Invalid("component scratch and output publication parents differ".to_string()));
    }
    let source_name = c_file_name(&scratch_root)?;
    let destination_name = c_file_name(evidence_dir)?;
    let mut options = fs::OpenOptions::new();
    options.read(true).custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let parent = options
        .open(source_parent)
        .map_err(|error| Error::io("opening no-follow component output parent", source_parent, error))?;
    use std::os::fd::AsRawFd;
    // SAFETY: both names are NUL-free basenames and both fds name the same no-follow parent.
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            parent.as_raw_fd(),
            source_name.as_ptr(),
            parent.as_raw_fd(),
            destination_name.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result != 0 {
        return Err(Error::io(
            "publishing component output without replacement",
            evidence_dir,
            std::io::Error::last_os_error(),
        ));
    }
    parent
        .sync_all()
        .map_err(|error| Error::io("syncing component output parent", source_parent, error))?;
    debug_assert!(evidence_dir.is_dir());
    debug_assert!(!scratch_root.exists());
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn publish_scratch(_scratch_root: PathBuf, _evidence_dir: &Path) -> Result<(), Error> {
    Err(Error::Blocked(
        "atomic no-replace component publication requires Linux renameat2(RENAME_NOREPLACE)".to_string(),
    ))
}

#[cfg(target_os = "linux")]
fn c_file_name(path: &Path) -> Result<CString, Error> {
    let name = path
        .file_name()
        .ok_or_else(|| Error::Invalid(format!("publication path has no file name: {}", path.display())))?;
    CString::new(name.as_bytes())
        .map_err(|_| Error::Invalid(format!("publication file name contains NUL: {}", path.display())))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::os::unix::fs::symlink;

    use super::publish_scratch;

    #[test]
    fn publication_never_replaces_existing_directory_or_symlink() {
        let parent = tempfile::tempdir().unwrap();
        let source = parent.path().join("source");
        let destination = parent.path().join("destination");
        std::fs::create_dir(&source).unwrap();
        std::fs::write(source.join("artifact"), b"new").unwrap();
        std::fs::create_dir(&destination).unwrap();
        let error = publish_scratch(source.clone(), &destination).expect_err("existing directory must not be replaced");
        assert!(error.to_string().contains("without replacement"));
        assert!(source.join("artifact").is_file());
        assert!(destination.is_dir());

        std::fs::remove_dir(&destination).unwrap();
        let target = parent.path().join("target");
        std::fs::create_dir(&target).unwrap();
        symlink(&target, &destination).unwrap();
        let error =
            publish_scratch(source.clone(), &destination).expect_err("symlink destination must not be replaced");
        assert!(error.to_string().contains("without replacement"));
        assert!(std::fs::symlink_metadata(&destination).unwrap().file_type().is_symlink());
        assert!(source.join("artifact").is_file());
    }
}
