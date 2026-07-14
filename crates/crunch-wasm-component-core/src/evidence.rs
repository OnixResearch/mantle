use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::AotAdmission;
use crate::Blake3Identity;
use crate::ComponentBlocker;
use crate::ComponentBuildReport;
use crate::ComponentStageKind;
use crate::ComponentStageStatus;
use crate::MaterializationBundle;
use crate::StageReportNode;
use crate::StoreObject;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::verify_component_report;
use crate::verify_materialization_bundle;

pub const COMPONENT_ARTIFACT_ATTESTATION_SCHEMA: &str = "mantle-wasm-component-artifact-attestation-v1";
pub const COMPONENT_RELEASE_BINDING_SCHEMA: &str = "mantle-wasm-component-release-binding-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentEvidenceRequest {
    pub bundle: MaterializationBundle,
    pub bundle_object: StoreObject,
    pub report: ComponentBuildReport,
    pub octet_profile_blake3: Blake3Identity,
    pub octet_cohort_blake3: Blake3Identity,
    pub octet_report: StoreObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentArtifactAttestation {
    pub schema: String,
    pub materialization_bundle_blake3: Blake3Identity,
    pub materialization_bundle_object: StoreObject,
    pub component_report_blake3: Blake3Identity,
    pub stage_node_blake3: Vec<Blake3Identity>,
    pub final_portable: StoreObject,
    pub octet_profile_blake3: Blake3Identity,
    pub octet_cohort_blake3: Blake3Identity,
    pub octet_report: StoreObject,
    pub aot: Option<AotAdmission>,
    pub non_claims: Vec<String>,
    pub attestation_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentReleaseBinding {
    pub schema: String,
    pub materialization_bundle_blake3: Blake3Identity,
    pub materialization_bundle_object: StoreObject,
    pub artifact_attestation_blake3: Blake3Identity,
    pub portable: StoreObject,
    pub target_specific_native: Option<AotAdmission>,
    pub octet_profile_blake3: Blake3Identity,
    pub octet_cohort_blake3: Blake3Identity,
    pub octet_report_blake3: Blake3Identity,
    pub release_eligible: bool,
    pub non_claims: Vec<String>,
    pub release_binding_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentEvidenceResult {
    pub artifact_attestation: Option<ComponentArtifactAttestation>,
    pub release_binding: Option<ComponentReleaseBinding>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Serialize)]
struct AttestationIdentityInput {
    schema: String,
    materialization_bundle_blake3: Blake3Identity,
    materialization_bundle_object: StoreObject,
    component_report_blake3: Blake3Identity,
    stage_node_blake3: Vec<Blake3Identity>,
    final_portable: StoreObject,
    octet_profile_blake3: Blake3Identity,
    octet_cohort_blake3: Blake3Identity,
    octet_report: StoreObject,
    aot: Option<AotAdmission>,
    non_claims: Vec<String>,
}

#[derive(Serialize)]
struct ReleaseIdentityInput {
    schema: String,
    materialization_bundle_blake3: Blake3Identity,
    materialization_bundle_object: StoreObject,
    artifact_attestation_blake3: Blake3Identity,
    portable: StoreObject,
    target_specific_native: Option<AotAdmission>,
    octet_profile_blake3: Blake3Identity,
    octet_cohort_blake3: Blake3Identity,
    octet_report_blake3: Blake3Identity,
    release_eligible: bool,
    non_claims: Vec<String>,
}

pub fn build_component_evidence(request: ComponentEvidenceRequest) -> ComponentEvidenceResult {
    let mut blockers = validate_request(&request);
    if !blockers.is_empty() {
        return ComponentEvidenceResult {
            artifact_attestation: None,
            release_binding: None,
            blockers,
        };
    }
    let attestation = match artifact_attestation(&request) {
        Ok(attestation) => attestation,
        Err(item) => {
            blockers.push(item);
            return ComponentEvidenceResult {
                artifact_attestation: None,
                release_binding: None,
                blockers,
            };
        }
    };
    let release = match release_binding(&request, &attestation) {
        Ok(release) => release,
        Err(item) => {
            blockers.push(item);
            return ComponentEvidenceResult {
                artifact_attestation: None,
                release_binding: None,
                blockers,
            };
        }
    };
    debug_assert!(!attestation.stage_node_blake3.is_empty());
    debug_assert!(!release.release_eligible);
    ComponentEvidenceResult {
        artifact_attestation: Some(attestation),
        release_binding: Some(release),
        blockers: Vec::new(),
    }
}

fn validate_request(request: &ComponentEvidenceRequest) -> Vec<ComponentBlocker> {
    let mut blockers = Vec::new();
    let bundle_result = verify_materialization_bundle(request.bundle.clone());
    blockers.extend(bundle_result.blockers);
    blockers.extend(verify_component_report(request.report.clone()).blockers);
    validate_object(&request.bundle_object, "materialization-bundle", &mut blockers);
    validate_object(&request.octet_report, "octet-report", &mut blockers);
    if !blockers.is_empty() {
        return blockers;
    }
    validate_octet_stage(request, &mut blockers);
    validate_materialization_stage(request, &mut blockers);
    validate_optional_stage(request, ComponentStageKind::Aot, request.bundle.aot.is_some(), &mut blockers);
    validate_optional_stage(request, ComponentStageKind::Wizer, request.bundle.wizer.is_some(), &mut blockers);
    validate_optional_stage(
        request,
        ComponentStageKind::Componentization,
        request.bundle.wizer.is_some(),
        &mut blockers,
    );
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    debug_assert!(blockers.iter().all(|item| !item.message.is_empty()));
    blockers
}

fn validate_octet_stage(request: &ComponentEvidenceRequest, blockers: &mut Vec<ComponentBlocker>) {
    let Some(node) = unique_stage(&request.report.nodes, ComponentStageKind::OctetValidation, blockers) else {
        return;
    };
    if !octet_stage_matches(request, node) {
        blockers.push(blocker(
            "component-attestation-octet-mismatch",
            "octet-validation",
            "Octet stage, exact report object, profile, and materialization bundle do not agree",
        ));
    }
}

fn octet_stage_matches(request: &ComponentEvidenceRequest, node: &StageReportNode) -> bool {
    if node.status != ComponentStageStatus::Succeeded {
        return false;
    }
    if node.artifact.as_ref() != Some(&request.octet_report) {
        return false;
    }
    if node.profile_identity_blake3.as_ref() != Some(&request.octet_profile_blake3) {
        return false;
    }
    if request.bundle.expected_octet_profile_blake3 != request.octet_profile_blake3 {
        return false;
    }
    debug_assert_eq!(node.artifact.as_ref(), Some(&request.octet_report));
    debug_assert_eq!(node.profile_identity_blake3.as_ref(), Some(&request.octet_profile_blake3));
    true
}

fn validate_materialization_stage(request: &ComponentEvidenceRequest, blockers: &mut Vec<ComponentBlocker>) {
    let Some(node) = unique_stage(&request.report.nodes, ComponentStageKind::MaterializationBundle, blockers) else {
        return;
    };
    if node.status != ComponentStageStatus::Succeeded || node.artifact.as_ref() != Some(&request.bundle_object) {
        blockers.push(blocker(
            "component-attestation-bundle-mismatch",
            "materialization-bundle",
            "materialization stage does not identify the exact verified bundle object",
        ));
    }
}

fn validate_optional_stage(
    request: &ComponentEvidenceRequest,
    kind: ComponentStageKind,
    required: bool,
    blockers: &mut Vec<ComponentBlocker>,
) {
    let matches: Vec<&StageReportNode> = request.report.nodes.iter().filter(|node| node.kind == kind).collect();
    if required && (matches.len() != 1 || matches[0].status != ComponentStageStatus::Succeeded) {
        blockers.push(blocker(
            "component-attestation-optional-stage-mismatch",
            &stage_label(kind),
            "materialization optional output lacks one successful matching stage node",
        ));
    }
}

fn unique_stage<'a>(
    nodes: &'a [StageReportNode],
    kind: ComponentStageKind,
    blockers: &mut Vec<ComponentBlocker>,
) -> Option<&'a StageReportNode> {
    let matches: Vec<&StageReportNode> = nodes.iter().filter(|node| node.kind == kind).collect();
    if matches.len() != 1 {
        blockers.push(blocker(
            "component-attestation-stage-cardinality",
            &stage_label(kind),
            "component evidence requires exactly one matching stage node",
        ));
        return None;
    }
    matches.first().copied()
}

fn artifact_attestation(request: &ComponentEvidenceRequest) -> Result<ComponentArtifactAttestation, ComponentBlocker> {
    let mut non_claims = request.bundle.non_claims.clone();
    non_claims.sort();
    let stage_node_blake3 = request.report.nodes.iter().map(|node| node.identity_blake3.clone()).collect();
    let input = AttestationIdentityInput {
        schema: String::from(COMPONENT_ARTIFACT_ATTESTATION_SCHEMA),
        materialization_bundle_blake3: request.bundle.bundle_identity_blake3.clone(),
        materialization_bundle_object: request.bundle_object.clone(),
        component_report_blake3: request.report.report_identity_blake3.clone(),
        stage_node_blake3,
        final_portable: request.bundle.final_portable.clone(),
        octet_profile_blake3: request.octet_profile_blake3.clone(),
        octet_cohort_blake3: request.octet_cohort_blake3.clone(),
        octet_report: request.octet_report.clone(),
        aot: request.bundle.aot.clone(),
        non_claims,
    };
    let identity = canonical_identity(&input).map_err(|_| identity_blocker("component-attestation-identity-failed"))?;
    let attestation = ComponentArtifactAttestation {
        schema: input.schema,
        materialization_bundle_blake3: input.materialization_bundle_blake3,
        materialization_bundle_object: input.materialization_bundle_object,
        component_report_blake3: input.component_report_blake3,
        stage_node_blake3: input.stage_node_blake3,
        final_portable: input.final_portable,
        octet_profile_blake3: input.octet_profile_blake3,
        octet_cohort_blake3: input.octet_cohort_blake3,
        octet_report: input.octet_report,
        aot: input.aot,
        non_claims: input.non_claims,
        attestation_blake3: identity,
    };
    debug_assert!(!attestation.stage_node_blake3.is_empty());
    debug_assert!(!attestation.non_claims.is_empty());
    Ok(attestation)
}

fn release_binding(
    request: &ComponentEvidenceRequest,
    attestation: &ComponentArtifactAttestation,
) -> Result<ComponentReleaseBinding, ComponentBlocker> {
    let input = ReleaseIdentityInput {
        schema: String::from(COMPONENT_RELEASE_BINDING_SCHEMA),
        materialization_bundle_blake3: request.bundle.bundle_identity_blake3.clone(),
        materialization_bundle_object: request.bundle_object.clone(),
        artifact_attestation_blake3: attestation.attestation_blake3.clone(),
        portable: request.bundle.final_portable.clone(),
        target_specific_native: request.bundle.aot.clone(),
        octet_profile_blake3: request.octet_profile_blake3.clone(),
        octet_cohort_blake3: request.octet_cohort_blake3.clone(),
        octet_report_blake3: request.octet_report.digest_blake3.clone(),
        release_eligible: false,
        non_claims: request.bundle.non_claims.clone(),
    };
    let identity = canonical_identity(&input).map_err(|_| identity_blocker("component-release-identity-failed"))?;
    let release = ComponentReleaseBinding {
        schema: input.schema,
        materialization_bundle_blake3: input.materialization_bundle_blake3,
        materialization_bundle_object: input.materialization_bundle_object,
        artifact_attestation_blake3: input.artifact_attestation_blake3,
        portable: input.portable,
        target_specific_native: input.target_specific_native,
        octet_profile_blake3: input.octet_profile_blake3,
        octet_cohort_blake3: input.octet_cohort_blake3,
        octet_report_blake3: input.octet_report_blake3,
        release_eligible: input.release_eligible,
        non_claims: input.non_claims,
        release_binding_blake3: identity,
    };
    debug_assert!(!release.release_eligible);
    debug_assert!(!release.non_claims.is_empty());
    Ok(release)
}

fn validate_object(object: &StoreObject, subject: &str, blockers: &mut Vec<ComponentBlocker>) {
    if object.size_bytes == 0 || !object.logical_path.starts_with('/') {
        blockers.push(blocker(
            "invalid-component-evidence-object",
            subject,
            "component evidence objects require absolute locators and positive byte sizes",
        ));
    }
}

fn identity_blocker(code: &str) -> ComponentBlocker {
    blocker(code, "component-evidence", "component evidence could not be canonically identified")
}

fn stage_label(kind: ComponentStageKind) -> String {
    let label = match kind {
        ComponentStageKind::PackageResolution => "package-resolution",
        ComponentStageKind::Lock => "lock",
        ComponentStageKind::BindingGeneration => "binding-generation",
        ComponentStageKind::Compilation => "compilation",
        ComponentStageKind::Componentization => "componentization",
        ComponentStageKind::Composition => "composition",
        ComponentStageKind::Virtualization => "virtualization",
        ComponentStageKind::MetadataNormalization => "metadata-normalization",
        ComponentStageKind::BuildValidation => "build-validation",
        ComponentStageKind::OctetValidation => "octet-validation",
        ComponentStageKind::Wizer => "wizer",
        ComponentStageKind::Aot => "aot",
        ComponentStageKind::MaterializationBundle => "materialization-bundle",
    };
    String::from(label)
}
