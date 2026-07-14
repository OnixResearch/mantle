use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::ComponentEvidenceRequest;
use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::build_component_evidence;
use crunch_wasm_component_core::verify_component_report;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::Error;
use crate::PIPELINE_EXECUTION_REPORT_SCHEMA;
use crate::PipelineArtifact;
use crate::PipelineExecutionReport;
use crate::verify_materialization_bundle_files;

const MATERIALIZATION_BUNDLE_ROLE: &str = "component-materialization-bundle";
const COMPONENT_ATTESTATION_ROLE: &str = "component-artifact-attestation";
const COMPONENT_RELEASE_BINDING_ROLE: &str = "component-release-binding";

pub fn verify_pipeline_execution_report_files(report: &PipelineExecutionReport) -> Result<(), Error> {
    verify_report_identity(report)?;
    verify_published_artifacts(&report.artifacts)?;
    let bundle = report
        .materialization_bundle
        .as_ref()
        .ok_or_else(|| Error::Invalid("successful component report omitted its materialization bundle".to_string()))?;
    verify_materialization_bundle_files(bundle)?;
    verify_evidence_sidecars(report, bundle)?;
    debug_assert_eq!(report.final_status, "succeeded");
    debug_assert!(report.blockers.is_empty());
    Ok(())
}

fn verify_report_identity(report: &PipelineExecutionReport) -> Result<(), Error> {
    if report.schema != PIPELINE_EXECUTION_REPORT_SCHEMA {
        return Err(Error::Invalid("component execution report schema is unsupported".to_string()));
    }
    if report.final_status != "succeeded" || !report.blockers.is_empty() {
        return Err(Error::Invalid("component execution report is not an admitted successful report".to_string()));
    }
    let input = ExecutionReportIdentityInput::from(report);
    let bytes = serde_json::to_vec(&input)
        .map_err(|error| Error::Invalid(format!("serializing component report verification input: {error}")))?;
    let observed = Blake3Identity::from_slice(&bytes);
    if observed != report.report_blake3 {
        return Err(Error::Invalid("component execution report identity drifted".to_string()));
    }
    let component = report
        .component_report
        .clone()
        .ok_or_else(|| Error::Invalid("successful execution report omitted its component stage graph".to_string()))?;
    let verified = verify_component_report(component.clone());
    if verified.report.as_ref() != Some(&component) || !verified.blockers.is_empty() {
        return Err(crate::preflight::core_blockers("component report verification", &verified.blockers));
    }
    debug_assert_eq!(observed, report.report_blake3);
    debug_assert!(report.component_report.is_some());
    Ok(())
}

fn verify_published_artifacts(artifacts: &[PipelineArtifact]) -> Result<(), Error> {
    let mut unique = BTreeMap::new();
    for artifact in artifacts {
        if let Some(prior) = unique.insert(artifact.path.clone(), artifact)
            && (prior.digest_blake3 != artifact.digest_blake3 || prior.size_bytes != artifact.size_bytes)
        {
            return Err(Error::Invalid(format!(
                "published artifact locator has conflicting identities: {}",
                artifact.path
            )));
        }
    }
    for artifact in unique.values() {
        let path = Path::new(&artifact.path);
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| Error::io("reading published component artifact metadata", path, error))?;
        if !metadata.file_type().is_file()
            || metadata.len() != artifact.size_bytes
            || crate::toolchain::hash_file_bounded(path)? != artifact.digest_blake3
        {
            return Err(Error::Invalid(format!("published component artifact bytes drifted: {}", artifact.path)));
        }
    }
    debug_assert!(!unique.is_empty());
    debug_assert!(unique.values().all(|artifact| artifact.size_bytes > 0));
    Ok(())
}

fn verify_evidence_sidecars(report: &PipelineExecutionReport, bundle: &MaterializationBundle) -> Result<(), Error> {
    let bundle_object = artifact_object(report, MATERIALIZATION_BUNDLE_ROLE)?;
    let parsed_bundle: MaterializationBundle = read_json_object(&bundle_object, "materialization bundle")?;
    if parsed_bundle != *bundle {
        return Err(Error::Invalid("materialization bundle sidecar differs from the execution report".to_string()));
    }
    let attestation = report
        .component_attestation
        .clone()
        .ok_or_else(|| Error::Invalid("successful report omitted its component attestation".to_string()))?;
    let release = report
        .release_binding
        .clone()
        .ok_or_else(|| Error::Invalid("successful report omitted its component release binding".to_string()))?;
    let parsed_attestation: crunch_wasm_component_core::ComponentArtifactAttestation =
        read_json_object(&artifact_object(report, COMPONENT_ATTESTATION_ROLE)?, "attestation")?;
    let parsed_release: crunch_wasm_component_core::ComponentReleaseBinding =
        read_json_object(&artifact_object(report, COMPONENT_RELEASE_BINDING_ROLE)?, "release binding")?;
    if parsed_attestation != attestation || parsed_release != release {
        return Err(Error::Invalid("component attestation or release sidecar differs from the report".to_string()));
    }
    reconstruct_component_evidence(report, bundle.clone(), bundle_object, attestation, release)
}

fn reconstruct_component_evidence(
    report: &PipelineExecutionReport,
    bundle: MaterializationBundle,
    bundle_object: StoreObject,
    expected_attestation: crunch_wasm_component_core::ComponentArtifactAttestation,
    expected_release: crunch_wasm_component_core::ComponentReleaseBinding,
) -> Result<(), Error> {
    let component_report = report
        .component_report
        .clone()
        .ok_or_else(|| Error::Invalid("component stage graph was absent during evidence reconstruction".to_string()))?;
    let octet = report.octet_validations.last().ok_or_else(|| {
        Error::Invalid("Octet evidence was absent during component evidence reconstruction".to_string())
    })?;
    let rebuilt = build_component_evidence(ComponentEvidenceRequest {
        bundle,
        bundle_object,
        report: component_report,
        octet_profile_blake3: octet.profile_blake3.clone(),
        octet_cohort_blake3: octet.cohort_blake3.clone(),
        octet_report: octet.receipt.clone(),
    });
    if rebuilt.artifact_attestation.as_ref() != Some(&expected_attestation)
        || rebuilt.release_binding.as_ref() != Some(&expected_release)
        || !rebuilt.blockers.is_empty()
    {
        return Err(crate::preflight::core_blockers("component evidence reconstruction", &rebuilt.blockers));
    }
    debug_assert_eq!(rebuilt.artifact_attestation, Some(expected_attestation));
    debug_assert_eq!(rebuilt.release_binding, Some(expected_release));
    Ok(())
}

fn artifact_object(report: &PipelineExecutionReport, role: &str) -> Result<StoreObject, Error> {
    let matches: Vec<&PipelineArtifact> = report.artifacts.iter().filter(|artifact| artifact.role == role).collect();
    if matches.len() != 1 {
        return Err(Error::Invalid(format!("component report requires one `{role}` artifact")));
    }
    let artifact = matches[0];
    if !artifact.path.starts_with('/') || artifact.size_bytes == 0 {
        return Err(Error::Invalid(format!("invalid `{role}` object metadata")));
    }
    Ok(StoreObject {
        logical_path: artifact.path.clone(),
        digest_blake3: artifact.digest_blake3.clone(),
        size_bytes: artifact.size_bytes,
    })
}

fn read_json_object<T: DeserializeOwned>(object: &StoreObject, label: &str) -> Result<T, Error> {
    let path = Path::new(&object.logical_path);
    let bytes = crate::files::read_source_file_bounded(path, object.size_bytes, label)?;
    serde_json::from_slice(&bytes).map_err(|error| Error::Invalid(format!("parsing {label}: {error}")))
}

#[derive(Serialize)]
struct ExecutionReportIdentityInput<'a> {
    schema: &'a str,
    request_blake3: &'a Blake3Identity,
    cohort_blake3: &'a Option<Blake3Identity>,
    stage_receipts: &'a [crate::ToolExecutionReceipt],
    artifacts: &'a [PipelineArtifact],
    blockers: &'a [crate::PipelineBlocker],
    octet_validations: &'a [crate::OctetValidationEvidence],
    component_report: &'a Option<crunch_wasm_component_core::ComponentBuildReport>,
    materialization_bundle: &'a Option<MaterializationBundle>,
    component_attestation: &'a Option<crunch_wasm_component_core::ComponentArtifactAttestation>,
    release_binding: &'a Option<crunch_wasm_component_core::ComponentReleaseBinding>,
    final_status: &'a str,
    non_claims: &'a [String],
}

impl<'a> From<&'a PipelineExecutionReport> for ExecutionReportIdentityInput<'a> {
    fn from(report: &'a PipelineExecutionReport) -> Self {
        Self {
            schema: &report.schema,
            request_blake3: &report.request_blake3,
            cohort_blake3: &report.cohort_blake3,
            stage_receipts: &report.stage_receipts,
            artifacts: &report.artifacts,
            blockers: &report.blockers,
            octet_validations: &report.octet_validations,
            component_report: &report.component_report,
            materialization_bundle: &report.materialization_bundle,
            component_attestation: &report.component_attestation,
            release_binding: &report.release_binding,
            final_status: &report.final_status,
            non_claims: &report.non_claims,
        }
    }
}
